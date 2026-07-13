#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

node - "${ROOT_DIR}" <<'NODE'
const fs = require("fs");
const path = require("path");

const root = path.resolve(process.argv[2]);
const templateRelative = "rust-gateway/resources/views/admin.html";
const stylesheetRelative = "rust-gateway/resources/admin/admin-console.css";
const scriptRelative = "rust-gateway/resources/admin/admin-console.js";
const webPagesRelative = "rust-gateway/src/web_pages.rs";
const routerRelative = "rust-gateway/src/router_support.rs";
const staticFilesRelative = "rust-gateway/src/static_files.rs";

const failures = [];
const warnings = [];

function fail(message) {
  failures.push(message);
}

function warn(message) {
  warnings.push(message);
}

function read(relative) {
  const absolute = path.join(root, relative);
  if (!fs.existsSync(absolute)) {
    fail(`${relative}: file is missing`);
    return "";
  }
  return fs.readFileSync(absolute, "utf8");
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function startTagWithId(source, id) {
  const pattern = new RegExp(
    `<([a-zA-Z][\\w:-]*)\\b(?=[^>]*\\sid\\s*=\\s*(?:"${escapeRegExp(id)}"|'${escapeRegExp(id)}'))[^>]*>`,
    "i"
  );
  return source.match(pattern)?.[0] || "";
}

function hasAttribute(tag, name, expected = null) {
  const attribute = new RegExp(
    `(?:^|\\s)${escapeRegExp(name)}(?=\\s|=|\\/?>)(?:\\s*=\\s*(?:"([^"]*)"|'([^']*)'|([^\\s>]+)))?`,
    "i"
  ).exec(tag);
  if (!attribute) return false;
  if (expected === null) return true;
  const value = attribute[1] ?? attribute[2] ?? attribute[3] ?? "";
  return value.toLowerCase() === expected.toLowerCase();
}

function functionBody(source, name) {
  const signature = new RegExp(`(?:pub(?:\\([^)]*\\))?\\s+)?(?:async\\s+)?fn\\s+${escapeRegExp(name)}\\b`);
  const match = signature.exec(source);
  if (!match) return "";
  const opening = source.indexOf("{", match.index + match[0].length);
  if (opening < 0) return "";
  let depth = 0;
  for (let index = opening; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(opening + 1, index);
    }
  }
  return "";
}

function javascriptFunctionBody(source, name) {
  const signature = new RegExp(`(?:async\\s+)?function\\s+${escapeRegExp(name)}\\s*\\([^)]*\\)\\s*\\{`);
  const match = signature.exec(source);
  if (!match) return "";
  const opening = match.index + match[0].lastIndexOf("{");
  if (opening < 0) return "";
  let depth = 0;
  for (let index = opening; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(opening + 1, index);
    }
  }
  return "";
}

function patternIndex(source, pattern) {
  const match = pattern.exec(source);
  return match ? match.index : -1;
}

const template = read(templateRelative);
const stylesheet = read(stylesheetRelative);
const script = read(scriptRelative);
const webPages = read(webPagesRelative);
const router = read(routerRelative);
const staticFiles = read(staticFilesRelative);

const stylesheetTag = template.match(/<link\b[^>]*\shref\s*=\s*(?:"\/\{\{\s*\$secure_path\s*\}\}\/assets\/admin-console\.css(?:\?[^">]*)?"|'\/\{\{\s*\$secure_path\s*\}\}\/assets\/admin-console\.css(?:\?[^'>]*)?')[^>]*>/i)?.[0] || "";
if (!stylesheetTag) {
  fail(`${templateRelative}: missing secure-path admin-console.css link`);
}

const scriptTag = template.match(/<script\b[^>]*\ssrc\s*=\s*(?:"\/\{\{\s*\$secure_path\s*\}\}\/assets\/admin-console\.js(?:\?[^">]*)?"|'\/\{\{\s*\$secure_path\s*\}\}\/assets\/admin-console\.js(?:\?[^'>]*)?')[^>]*>\s*<\/script>/i)?.[0] || "";
if (!scriptTag) {
  fail(`${templateRelative}: missing secure-path admin-console.js script`);
} else if (!hasAttribute(scriptTag, "defer")) {
  fail(`${templateRelative}: admin-console.js must use the defer attribute`);
}

for (const publicReference of [
  /\s(?:href|src)\s*=\s*["']\/assets\/admin-console\.(?:css|js)/i,
  /\s(?:href|src)\s*=\s*["']\/assets\/admin\/admin-console\.(?:css|js)/i,
]) {
  if (publicReference.test(template)) {
    fail(`${templateRelative}: admin console asset is referenced through the public /assets namespace`);
  }
}

for (const publicRelative of [
  "rust-gateway/resources/public/assets/admin-console.css",
  "rust-gateway/resources/public/assets/admin-console.js",
  "rust-gateway/resources/public/assets/admin/admin-console.css",
  "rust-gateway/resources/public/assets/admin/admin-console.js",
]) {
  if (fs.existsSync(path.join(root, publicRelative))) {
    fail(`${publicRelative}: private admin console asset must not live under resources/public`);
  }
}

if (stylesheet && stylesheet.trim().length < 1000) {
  fail(`${stylesheetRelative}: extracted stylesheet is unexpectedly small`);
}
if (script && script.trim().length < 1000) {
  fail(`${scriptRelative}: extracted application script is unexpectedly small`);
}

if (/<style\b[^>]*>[\s\S]*?<\/style>/i.test(template)) {
  fail(`${templateRelative}: inline style block remains after stylesheet extraction`);
}

const inlineScripts = [...template.matchAll(/<script\b(?![^>]*\ssrc\s*=)[^>]*>([\s\S]*?)<\/script>/gi)]
  .map((match) => match[1].trim())
  .filter(Boolean);
if (inlineScripts.length !== 1) {
  fail(`${templateRelative}: expected exactly one inline window.settings bootstrap, found ${inlineScripts.length}`);
} else {
  const bootstrap = inlineScripts[0];
  if (!/\bwindow\.settings\s*=/.test(bootstrap)) {
    fail(`${templateRelative}: the remaining inline script is not the window.settings bootstrap`);
  }
  if (bootstrap.length > 1200 || /\b(?:fetch|addEventListener|querySelector|getElementById)\s*\(/.test(bootstrap)) {
    fail(`${templateRelative}: application logic remains in the inline settings bootstrap`);
  }
}

const requiredIds = [
  "loginView",
  "adminLayout",
  "adminSidebar",
  "mainContent",
  "mobileNavToggle",
  "mobileNavBackdrop",
  "loginEmailInput",
  "loginPasswordInput",
  "loginSubmitBtn",
  "loginErrorText",
  "themeSwitcher",
  "themeDarkBtn",
  "themeLightBtn",
  "menuTabs",
  "primaryOpsMenuList",
  "currentViewTitle",
  "currentViewDescription",
  "authState",
  "currentAdminText",
  "refreshOverviewBtn",
  "logoutBtn",
  "panelOverview",
  "panelUsers",
  "panelApi",
  "panelOps",
  "overviewCards",
  "usersTableBody",
  "opsWorkspace",
  "toast",
  "actionDialog",
  "actionDialogTitle",
  "actionDialogMessage",
  "actionDialogReason",
  "actionDialogCancel",
  "actionDialogConfirm",
];

for (const id of requiredIds) {
  const matches = [...template.matchAll(new RegExp(`\\sid\\s*=\\s*(?:"${escapeRegExp(id)}"|'${escapeRegExp(id)}')`, "g"))];
  if (matches.length === 0) {
    fail(`${templateRelative}: required DOM id #${id} is missing`);
  } else if (matches.length > 1) {
    fail(`${templateRelative}: required DOM id #${id} is duplicated`);
  }
}

for (const id of ["globalNavSearch", "sidebarCloseBtn"]) {
  if (!startTagWithId(template, id)) {
    warn(`${templateRelative}: optional usability control #${id} is missing`);
  }
}

const skipLink = template.match(/<a\b[^>]*\shref\s*=\s*["']#mainContent["'][^>]*>/i)?.[0] || "";
if (!skipLink) {
  fail(`${templateRelative}: missing skip link targeting #mainContent`);
}

const mobileToggle = startTagWithId(template, "mobileNavToggle");
if (mobileToggle) {
  if (!/^<button\b/i.test(mobileToggle)) {
    fail(`${templateRelative}: #mobileNavToggle must be a button`);
  }
  if (!hasAttribute(mobileToggle, "aria-controls", "adminSidebar")) {
    fail(`${templateRelative}: #mobileNavToggle must control #adminSidebar`);
  }
  if (!hasAttribute(mobileToggle, "aria-expanded", "false")) {
    fail(`${templateRelative}: #mobileNavToggle must start with aria-expanded=false`);
  }
}

const toast = startTagWithId(template, "toast");
if (toast && (!hasAttribute(toast, "role", "status") || !hasAttribute(toast, "aria-live", "polite"))) {
  fail(`${templateRelative}: #toast must expose role=status and aria-live=polite`);
}

const loginError = startTagWithId(template, "loginErrorText");
if (loginError && (!hasAttribute(loginError, "role", "alert") || !hasAttribute(loginError, "aria-live", "assertive"))) {
  fail(`${templateRelative}: #loginErrorText must expose role=alert and aria-live=assertive`);
}

const dialog = startTagWithId(template, "actionDialog");
if (dialog) {
  if (!/^<dialog\b/i.test(dialog)) {
    fail(`${templateRelative}: #actionDialog must use the native dialog element`);
  }
  if (!hasAttribute(dialog, "aria-labelledby", "actionDialogTitle")) {
    fail(`${templateRelative}: #actionDialog must be labelled by #actionDialogTitle`);
  }
  if (!hasAttribute(dialog, "aria-describedby", "actionDialogMessage")) {
    fail(`${templateRelative}: #actionDialog must be described by #actionDialogMessage`);
  }
}

const expectedModules = [
  "security",
  "oauth",
  "site",
  "telegram",
  "plans",
  "payments",
  "notices",
  "tickets",
  "coupons",
  "giftcards",
  "riskreview",
  "plugins",
  "system",
  "traffic",
];
const groupRegistry = /\bconst\s+OPS_NAV_GROUPS\s*=\s*\[([\s\S]*?)\n\s*\];/.exec(script)?.[1] || "";
const declaredGroups = [...groupRegistry.matchAll(/\bkey\s*:\s*["']([a-z0-9_-]+)["']/g)].map((match) => match[1]);
if (declaredGroups.length < 5 || new Set(declaredGroups).size !== declaredGroups.length) {
  fail(`${scriptRelative}: OPS_NAV_GROUPS must define at least five unique workflow groups`);
}

const moduleRegistry = /\bconst\s+OPS_MODULES\s*=\s*\[([\s\S]*?)\n\s*\];/.exec(script)?.[1] || "";
if (!moduleRegistry) {
  fail(`${scriptRelative}: OPS_MODULES registry is missing`);
} else {
  const actualModules = [...moduleRegistry.matchAll(/\bkey\s*:\s*["']([a-z0-9_-]+)["']/g)].map((match) => match[1]);
  const duplicates = actualModules.filter((value, index) => actualModules.indexOf(value) !== index);
  const missing = expectedModules.filter((value) => !actualModules.includes(value));
  const unexpected = actualModules.filter((value) => !expectedModules.includes(value));
  if (actualModules.length !== expectedModules.length || missing.length || unexpected.length || duplicates.length) {
    fail(
      `${scriptRelative}: OPS_MODULES must contain exactly 14 keys; ` +
      `missing=[${missing.join(", ")}], unexpected=[${unexpected.join(", ")}], duplicates=[${[...new Set(duplicates)].join(", ")}]`
    );
  }

  const moduleEntries = [...moduleRegistry.matchAll(/\{([^{}]*\bkey\s*:\s*["'][a-z0-9_-]+["'][^{}]*)\}/g)]
    .map((match) => match[1]);
  for (const entry of moduleEntries) {
    const key = /\bkey\s*:\s*["']([a-z0-9_-]+)["']/.exec(entry)?.[1] || "unknown";
    const group = /\bgroup\s*:\s*["']([a-z0-9_-]+)["']/.exec(entry)?.[1] || "";
    if (!group || !declaredGroups.includes(group)) {
      fail(`${scriptRelative}: module ${key} references an undeclared workflow group`);
    }
    if (!/\brender\s*:\s*[a-zA-Z_$][\w$]*/.test(entry) || !/\bbind\s*:\s*[a-zA-Z_$][\w$]*/.test(entry)) {
      fail(`${scriptRelative}: module ${key} must declare render and bind handlers`);
    }
  }

  const loader = javascriptFunctionBody(script, "loadOpsModuleData");
  const loadedKeys = [...loader.matchAll(/\bmoduleKey\s*===?\s*["']([a-z0-9_-]+)["']/g)].map((match) => match[1]);
  const missingLoaders = expectedModules.filter((key) => !loadedKeys.includes(key));
  if (missingLoaders.length) {
    fail(`${scriptRelative}: loadOpsModuleData does not cover modules [${missingLoaders.join(", ")}]`);
  }
  if (!/\bMODULE_CONTROLLERS\s*\[\s*(?:key|moduleKey)\s*\]/.test(script) ||
      !/\bcontroller\.load\s*\(/.test(script) ||
      !/\bcontroller\.render\s*\(/.test(script) ||
      !/\bcontroller\.bind\s*\(/.test(script)) {
    fail(`${scriptRelative}: MODULE_CONTROLLERS is not the active load/render/bind dispatch path`);
  }
}

for (const forbidden of [
  ["window.prompt", /\bwindow\.prompt\s*\(/],
  ["window.confirm", /\bwindow\.confirm\s*\(/],
  ["bare prompt", /(^|[^.\w])prompt\s*\(/m],
  ["bare confirm", /(^|[^.\w])confirm\s*\(/m],
]) {
  if (forbidden[1].test(script)) {
    fail(`${scriptRelative}: ${forbidden[0]} remains; use #actionDialog`);
  }
}
if (!/\.showModal\s*\(/.test(script) || !/\bactionDialog\b/.test(script)) {
  fail(`${scriptRelative}: native #actionDialog workflow is not wired with showModal()`);
}

const readsHash = /(?:window\.)?location\.hash\b/.test(script);
const writesHash = /(?:window\.)?location\.hash\s*=|(?:function\s+syncViewRoute\b[\s\S]*?viewRoute\s*\([\s\S]*?(?:window\.)?history\.(?:replaceState|pushState)\s*\()/m.test(script);
const listensForHash = /(?:addEventListener\s*\(\s*["'](?:hashchange|popstate)["']|\.on(?:hashchange|popstate)\s*=)/.test(script);
const readsQuery = /URLSearchParams\s*\([^)]*(?:window\.)?location\.search|(?:window\.)?location\.searchParams\b/.test(script);
const writesQuery = /(?:window\.)?history\.(?:replaceState|pushState)\s*\(/.test(script);
const listensForPopstate = /(?:addEventListener\s*\(\s*["']popstate["']|\.onpopstate\s*=)/.test(script);
const hashRoundTrip = readsHash && writesHash && listensForHash;
const queryRoundTrip = readsQuery && writesQuery && listensForPopstate;
if (!(hashRoundTrip || queryRoundTrip)) {
  fail(`${scriptRelative}: deep links need one complete URL read/write/navigation-listener strategy`);
}

const requestHelper = javascriptFunctionBody(script, "request");
const loadUsers = javascriptFunctionBody(script, "loadUsers");
if (!loadUsers) {
  fail(`${scriptRelative}: loadUsers function is missing`);
} else {
  if (!/\bmethod\s*:\s*["']POST["']/.test(loadUsers) ||
      !/\burl\s*:\s*buildV2\s*\(\s*["']user\/fetch["']\s*\)/.test(loadUsers)) {
    fail(`${scriptRelative}: loadUsers must call user/fetch with POST`);
  }
  if (/\bmethod\s*:\s*["']GET["']/.test(loadUsers)) {
    fail(`${scriptRelative}: loadUsers must not call user/fetch with GET query parameters`);
  }
  if (!/\breadOnly\s*:\s*true\b/.test(loadUsers)) {
    fail(`${scriptRelative}: the POST user/fetch request must be marked readOnly`);
  }
  if (!/\bdata\s*:\s*\{[\s\S]*?\bcurrent\s*:[\s\S]*?\bpageSize\s*:[\s\S]*?\bfilter\s*:[\s\S]*?\bsort\s*:/m.test(loadUsers)) {
    fail(`${scriptRelative}: user/fetch must send pagination, filters, and sort in a JSON data object`);
  }
}
if (!requestHelper ||
    !/\bdata\s*!==\s*undefined\s*&&\s*method\s*!==\s*["']GET["']/.test(requestHelper) ||
    !/init\.headers\s*=\s*headers\s*\(\s*true\s*\)/.test(requestHelper) ||
    !/init\.body\s*=\s*JSON\.stringify\s*\(\s*data\s*\)/.test(requestHelper)) {
  fail(`${scriptRelative}: request() must serialize non-GET data as authenticated JSON`);
}
if (!requestHelper || !/!readOnly\s*&&\s*method\s*!==\s*["']GET["']/.test(requestHelper)) {
  fail(`${scriptRelative}: request() must keep read-only POST requests out of mutation invalidation`);
}

const sendApiRequest = javascriptFunctionBody(script, "sendApiRequest");
if (!sendApiRequest) {
  fail(`${scriptRelative}: sendApiRequest function is missing`);
} else {
  const parseCustomUrl = patternIndex(sendApiRequest, /new\s+URL\s*\(\s*endpoint\s*,\s*window\.location\.origin\s*\)/);
  const rejectForeignOrigin = patternIndex(sendApiRequest, /customUrl\.origin\s*!==\s*window\.location\.origin/);
  const rewriteToLocalPath = patternIndex(sendApiRequest, /url\s*=\s*`\$\{customUrl\.pathname\}\$\{customUrl\.search\}\$\{customUrl\.hash\}`/);
  const authenticatedRequest = patternIndex(sendApiRequest, /await\s+request\s*\(\s*\{/);
  if (parseCustomUrl < 0 || rejectForeignOrigin < parseCustomUrl ||
      rewriteToLocalPath < rejectForeignOrigin || authenticatedRequest < rewriteToLocalPath) {
    fail(`${scriptRelative}: custom API URLs must be origin-checked and reduced to a local path before request()`);
  }
  if (!/throw\s+new\s+Error\s*\([^)]*(?:当前站点|外部域名)/.test(sendApiRequest)) {
    fail(`${scriptRelative}: cross-origin custom API URLs must fail closed with an explicit error`);
  }
  if (/url\s*=\s*customUrl\.(?:href|toString\s*\(\s*\))/.test(sendApiRequest)) {
    fail(`${scriptRelative}: custom API URL must not retain an absolute href that could receive the admin token`);
  }
}

const opsBoolField = javascriptFunctionBody(script, "opsBoolField");
const opsBoolInput = opsBoolField.match(/<input\b[^>]*>/i)?.[0] || "";
if (!opsBoolField || !opsBoolInput) {
  fail(`${scriptRelative}: opsBoolField must render an input control`);
} else {
  if (!hasAttribute(opsBoolInput, "type", "checkbox") || !hasAttribute(opsBoolInput, "role", "switch")) {
    fail(`${scriptRelative}: opsBoolField must render a checkbox with role=switch`);
  }
  if (/<select\b/i.test(opsBoolField)) {
    fail(`${scriptRelative}: opsBoolField must not render the legacy select control`);
  }
  if (!/\bisOn\s*\(\s*value\s*\)[\s\S]*?["']checked["']/.test(opsBoolField)) {
    fail(`${scriptRelative}: opsBoolField must initialize the checkbox checked state from its value`);
  }
}
const readBooleanControl = javascriptFunctionBody(script, "readBoolControl");
const setBooleanControl = javascriptFunctionBody(script, "setInputValue");
if (!readBooleanControl || !/el\.type\s*===\s*["']checkbox["'][\s\S]*?el\.checked/.test(readBooleanControl)) {
  fail(`${scriptRelative}: boolean field reads must use checkbox.checked`);
}
if (!setBooleanControl || !/el\.type\s*===\s*["']checkbox["'][\s\S]*?el\.checked\s*=\s*isOn\s*\(\s*value\s*\)/.test(setBooleanControl)) {
  fail(`${scriptRelative}: boolean field writes must synchronize checkbox.checked`);
}

const clearToken = javascriptFunctionBody(script, "clearToken");
const activateView = javascriptFunctionBody(script, "activateView");
const openOpsModule = javascriptFunctionBody(script, "openOpsModule");
const bindOpsSystem = javascriptFunctionBody(script, "bindOpsSystem");
if (!/\busersRequestEpoch\s*:\s*0\b/.test(script) || !/\bnavigationEpoch\s*:\s*0\b/.test(script)) {
  fail(`${scriptRelative}: request and navigation epochs must be initialized in state`);
}
if (!clearToken ||
    !/state\.usersRequestEpoch\s*\+=\s*1/.test(clearToken) ||
    !/state\.navigationEpoch\s*\+=\s*1/.test(clearToken)) {
  fail(`${scriptRelative}: clearing authentication must invalidate in-flight user and navigation requests`);
}
if (!activateView || !/state\.navigationEpoch\s*\+=\s*1/.test(activateView)) {
  fail(`${scriptRelative}: changing the active view must advance navigationEpoch`);
}
if (openOpsModule) {
  const captureEpoch = patternIndex(openOpsModule, /const\s+epoch\s*=\s*\+\+state\.navigationEpoch/);
  const guardedPath = captureEpoch >= 0 ? openOpsModule.slice(captureEpoch) : "";
  const awaitLoadOffset = patternIndex(guardedPath, /await\s+controller\.load\s*\(/);
  const staleGuardOffset = patternIndex(guardedPath, /epoch\s*!==\s*state\.navigationEpoch[\s\S]*?state\.activeTab\s*!==\s*["']ops["'][\s\S]*?state\.activeOpsModule\s*!==\s*key/);
  const renderLoadedModuleOffset = patternIndex(guardedPath, /dom\.opsWorkspace\.innerHTML\s*=\s*renderOpsModule\s*\(\s*key\s*\)/);
  const catchOffset = patternIndex(guardedPath, /\}\s*catch\s*\([^)]*\)\s*\{/);
  const catchPath = catchOffset >= 0 ? guardedPath.slice(catchOffset) : "";
  const catchGuardOffset = patternIndex(catchPath, /epoch\s*!==\s*state\.navigationEpoch[\s\S]*?state\.activeTab\s*!==\s*["']ops["'][\s\S]*?state\.activeOpsModule\s*!==\s*key/);
  const renderErrorOffset = patternIndex(catchPath, /dom\.opsWorkspace\.innerHTML\s*=/);
  const awaitLoad = awaitLoadOffset < 0 ? -1 : captureEpoch + awaitLoadOffset;
  const staleGuard = staleGuardOffset < 0 ? -1 : captureEpoch + staleGuardOffset;
  const renderLoadedModule = renderLoadedModuleOffset < 0 ? -1 : captureEpoch + renderLoadedModuleOffset;
  const staleGuardCount = [...openOpsModule.matchAll(/epoch\s*!==\s*state\.navigationEpoch/g)].length;
  if (captureEpoch < 0 || awaitLoad < captureEpoch || staleGuard < awaitLoad ||
      renderLoadedModule < staleGuard || catchOffset < renderLoadedModuleOffset ||
      catchGuardOffset < 0 || renderErrorOffset < catchGuardOffset || staleGuardCount < 2) {
    fail(`${scriptRelative}: openOpsModule must reject stale success and error results before rendering`);
  }
} else {
  fail(`${scriptRelative}: openOpsModule function is missing`);
}
if (loadUsers) {
  const captureEpoch = patternIndex(loadUsers, /const\s+requestEpoch\s*=\s*\+\+state\.usersRequestEpoch/);
  const awaitUsers = patternIndex(loadUsers, /await\s+request\s*\(\s*\{/);
  const staleGuard = patternIndex(loadUsers, /requestEpoch\s*!==\s*state\.usersRequestEpoch/);
  const writeUsers = patternIndex(loadUsers, /state\.users\s*=/);
  const catchOffset = patternIndex(loadUsers, /\}\s*catch\s*\([^)]*\)\s*\{/);
  const catchPath = catchOffset >= 0 ? loadUsers.slice(catchOffset) : "";
  const catchGuardOffset = patternIndex(catchPath, /requestEpoch\s*!==\s*state\.usersRequestEpoch/);
  const writeUserErrorOffset = patternIndex(catchPath, /state\.usersLoaded\s*=\s*false/);
  const staleGuardCount = [...loadUsers.matchAll(/requestEpoch\s*!==\s*state\.usersRequestEpoch/g)].length;
  if (captureEpoch < 0 || awaitUsers < captureEpoch || staleGuard < awaitUsers ||
      writeUsers < staleGuard || catchOffset < writeUsers || catchGuardOffset < 0 ||
      writeUserErrorOffset < catchGuardOffset || staleGuardCount < 2 ||
      !/requestEpoch\s*===\s*state\.usersRequestEpoch[\s\S]*?state\.usersLoading\s*=\s*false/.test(loadUsers)) {
    fail(`${scriptRelative}: loadUsers must ignore stale success/error/finally paths using usersRequestEpoch`);
  }
}
if (bindOpsSystem) {
  const captureEpoch = patternIndex(bindOpsSystem, /const\s+requestEpoch\s*=\s*state\.navigationEpoch/);
  const awaitLogs = patternIndex(bindOpsSystem, /await\s+request\s*\(\s*\{[\s\S]*?system\/getSystemLog/);
  const staleGuard = patternIndex(bindOpsSystem, /requestEpoch\s*!==\s*state\.navigationEpoch[\s\S]*?state\.activeOpsModule\s*!==\s*["']system["']/);
  const renderLogs = patternIndex(bindOpsSystem, /dom\.opsWorkspace\.innerHTML\s*=\s*renderOpsSystem\s*\(\s*\)/);
  if (captureEpoch < 0 || awaitLogs < captureEpoch || staleGuard < awaitLogs || renderLogs < staleGuard) {
    fail(`${scriptRelative}: asynchronous system-log rendering must be guarded by navigationEpoch`);
  }
} else {
  fail(`${scriptRelative}: bindOpsSystem function is missing`);
}

for (const id of [
  "adminSidebar",
  "mobileNavToggle",
  "mobileNavBackdrop",
  "currentViewTitle",
  "currentViewDescription",
  "actionDialog",
]) {
  if (!new RegExp(`(?:getElementById\\(\\s*["']${escapeRegExp(id)}["']|#${escapeRegExp(id)}\\b)`).test(script)) {
    fail(`${scriptRelative}: UI behavior does not reference #${id}`);
  }
}
const currentAdminTag = startTagWithId(template, "currentAdminText");
if (currentAdminTag && !/^<(?:input|textarea|select)\b/i.test(currentAdminTag)) {
  if (/\bdom\.currentAdminText(?:\?\.)?\.value\b/.test(script)) {
    fail(`${scriptRelative}: #currentAdminText is a text element but the script reads or writes .value`);
  }
  if (!/\bdom\.currentAdminText(?:\?\.)?\.textContent\b/.test(script)) {
    fail(`${scriptRelative}: #currentAdminText is not updated through textContent`);
  }
}
if (!/\baria-expanded\b/.test(script)) {
  fail(`${scriptRelative}: mobile drawer does not synchronize aria-expanded`);
}
if (!/\.key\s*===?\s*["']Escape["']|["']Escape["']\s*===?\s*[\w$.?]+\.key/.test(script)) {
  fail(`${scriptRelative}: mobile drawer/dialog does not expose Escape-key handling`);
}

const responsiveBreakpoints = [...stylesheet.matchAll(/@media\s*\(\s*max-width\s*:\s*(\d+)px\s*\)/gi)]
  .map((match) => Number(match[1]));
if (!responsiveBreakpoints.some((value) => value >= 700 && value <= 1024)) {
  fail(`${stylesheetRelative}: missing a mobile/tablet drawer breakpoint between 700px and 1024px`);
}
if (!/(?:\.nav-open|\.is-open|\.drawer-open|\.sidebar-open|#adminSidebar\.(?:open|active)|\.sidebar\.(?:open|active)|\[data-(?:nav|drawer)-open)/.test(stylesheet)) {
  fail(`${stylesheetRelative}: missing an explicit open state for the mobile drawer`);
}
if (!/@media\s*\(\s*prefers-reduced-motion\s*:\s*reduce\s*\)/i.test(stylesheet)) {
  fail(`${stylesheetRelative}: missing prefers-reduced-motion handling`);
}
if (!/:focus-visible\b/.test(stylesheet)) {
  fail(`${stylesheetRelative}: missing visible keyboard focus styles`);
}

for (const [assetName, handlerName] of [
  ["admin-console.css", "admin_console_stylesheet"],
  ["admin-console.js", "admin_console_script"],
]) {
  const routePattern = new RegExp(
    `\\.route\\(\\s*["']\\/\\{admin_path\\}\\/assets\\/${escapeRegExp(assetName)}["']\\s*,\\s*get\\(\\s*web_pages::${handlerName}\\s*\\)\\s*,?\\s*\\)`,
    "s"
  );
  if (!routePattern.test(router)) {
    fail(`${routerRelative}: missing exact secure-path route for ${assetName}`);
  }

  const handler = functionBody(webPages, handlerName);
  if (!handler) {
    fail(`${webPagesRelative}: handler ${handlerName} is missing`);
  } else if (!/build_admin_console_asset_response/.test(handler)) {
    fail(`${webPagesRelative}: ${handlerName} must use the shared private asset response builder`);
  }
}

if (!new RegExp(`include_str!\\(\\s*["'][^"']*${escapeRegExp(stylesheetRelative.split("/").slice(-3).join("/"))}["']\\s*\\)`).test(webPages)) {
  fail(`${webPagesRelative}: admin-console.css is not embedded from the non-public admin resource directory`);
}
if (!new RegExp(`include_str!\\(\\s*["'][^"']*${escapeRegExp(scriptRelative.split("/").slice(-3).join("/"))}["']\\s*\\)`).test(webPages)) {
  fail(`${webPagesRelative}: admin-console.js is not embedded from the non-public admin resource directory`);
}

const assetBuilder = functionBody(webPages, "build_admin_console_asset_response");
if (!assetBuilder) {
  fail(`${webPagesRelative}: shared build_admin_console_asset_response helper is missing`);
} else {
  if (!/validate_secure_path\s*\(/.test(assetBuilder)) {
    fail(`${webPagesRelative}: private asset response builder does not validate the secure admin path`);
  }
  if (!/admin_console_asset_response\s*\(/.test(assetBuilder)) {
    fail(`${webPagesRelative}: private asset builder does not delegate to the typed response helper`);
  }
}

const assetResponse = functionBody(webPages, "admin_console_asset_response");
if (!assetResponse) {
  fail(`${webPagesRelative}: admin_console_asset_response helper is missing`);
} else {
  if (!/Cache-Control["']?\s*,\s*["'][^"']*(?:private|no-store)/i.test(assetResponse)) {
    fail(`${webPagesRelative}: private asset response does not set private/no-store cache control`);
  }
  if (!/X-Content-Type-Options["']?\s*,\s*["']nosniff/i.test(assetResponse)) {
    fail(`${webPagesRelative}: private asset response does not set nosniff`);
  }
  if (!/insert_privacy_headers\s*\(/.test(assetResponse)) {
    fail(`${webPagesRelative}: private asset response does not apply privacy headers`);
  }
}

const publicAssetHandler = functionBody(staticFiles, "assets_file");
const privateAssetGuard = functionBody(staticFiles, "is_private_admin_asset_path");
const publicGuardCall = patternIndex(publicAssetHandler, /is_private_admin_asset_path\s*\(/);
const publicGuardNotFound = patternIndex(publicAssetHandler, /StatusCode::NOT_FOUND/);
const publicAssetServe = patternIndex(publicAssetHandler, /serve_static_under\s*\(/);
if (!publicAssetHandler || publicGuardCall < 0 || publicGuardNotFound < publicGuardCall ||
    publicAssetServe < publicGuardNotFound) {
  fail(`${staticFilesRelative}: public asset handler does not reject the private admin prefix`);
}
if (!privateAssetGuard ||
    !/normalize_relative_path\s*\(\s*raw_path\s*\)[\s\S]*?components\s*\(\s*\)\.(?:next|find_map)\s*\([\s\S]*?eq_ignore_ascii_case\s*\(\s*["']admin["']\s*\)/.test(privateAssetGuard)) {
  fail(`${staticFilesRelative}: private admin asset prefix guard is missing or case-sensitive`);
}
const publicAssetGuardTest = functionBody(staticFiles, "public_assets_reject_private_admin_prefix");
if (!publicAssetGuardTest ||
    !/["']admin\/index\.html["']/.test(publicAssetGuardTest) ||
    !/["']\/ADMIN\/assets\/vendor\.js["']/.test(publicAssetGuardTest) ||
    !/["']administrator\/app\.js["']/.test(publicAssetGuardTest) ||
    !/["']admin-console\.js["']/.test(publicAssetGuardTest)) {
  fail(`${staticFilesRelative}: missing regression test for the public /assets/admin prefix`);
}
if (!/fn\s+admin_console_assets_are_private_and_typed\b/.test(webPages)) {
  fail(`${webPagesRelative}: missing regression test for private typed admin console assets`);
}

for (const message of warnings) {
  console.warn(`WARN: ${message}`);
}

if (failures.length > 0) {
  for (const message of failures) {
    console.error(`FAIL: ${message}`);
  }
  console.error(`Admin console UI verification failed with ${failures.length} finding(s).`);
  process.exit(1);
}

console.log(`Admin console UI verification passed (${requiredIds.length} required DOM ids, ${expectedModules.length} modules, request security and race guards).`);
NODE
