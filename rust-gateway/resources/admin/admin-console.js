    (() => {
      const settings = window.settings || {};
      const createOpsCache = () => ({
        config: null,
        plans: [],
        groups: [],
        payments: [],
        paymentMethods: [],
        notices: [],
        tickets: [],
        coupons: [],
        giftTemplates: [],
        giftCodes: [],
        giftTypes: {},
        giftStats: null,
        plugins: [],
        systemStatus: null,
        queueStats: null,
        systemLogs: null,
        failedJobs: null,
        riskReviews: [],
        riskReviewPagination: null,
        riskReviewPage: 1,
        banRecords: [],
        banRecordPagination: null,
        banRecordPage: 1,
        trafficStats: null,
        trafficLogs: [],
        trafficPagination: null,
        trafficDays: 30
      });

      const state = {
        token: "",
        currentAdmin: "",
        activeTab: "overview",
        adminTheme: "light",
        loginCommConfig: null,
        loginCommConfigPromise: null,
        loginCaptchaProvider: "",
        loginCaptchaToken: "",
        loginCaptchaWidgetId: null,
        loginCaptchaScriptPromise: null,
        loginCaptchaScriptSrc: "",
        loginCaptchaV3Ready: false,
        loginPowProof: null,
        loginPowPromise: null,
        usersPage: 1,
        usersLastPage: 1,
        usersPageSize: 20,
        usersTotal: 0,
        usersLoaded: false,
        usersLoading: false,
        usersRequestEpoch: 0,
        usersSearchField: "email",
        usersSearch: "",
        usersStatus: "all",
        users: [],
        overviewLoaded: false,
        overviewRefreshTimer: null,
        overviewCountdownTimer: null,
        overviewNextRefreshIn: 0,
        overviewRefreshIntervalSeconds: 20,
        activeOpsModule: "security",
        loadedOpsModules: new Set(),
        navigationEpoch: 0,
        opsDirty: false,
        opsDirtyRevision: 0,
        opsDirtyScopes: new Set(),
        opsDirtyScopeRevisions: new WeakMap(),
        actionDialogResolve: null,
        opsCache: createOpsCache()
      };

      const dom = {
        loginView: document.getElementById("loginView"),
        adminLayout: document.getElementById("adminLayout"),
        loginEmailInput: document.getElementById("loginEmailInput"),
        loginPasswordInput: document.getElementById("loginPasswordInput"),
        loginCaptchaField: document.getElementById("loginCaptchaField"),
        loginCaptchaWidget: document.getElementById("loginCaptchaWidget"),
        loginCaptchaHint: document.getElementById("loginCaptchaHint"),
        loginPowHint: document.getElementById("loginPowHint"),
        loginSubmitBtn: document.getElementById("loginSubmitBtn"),
        loginErrorText: document.getElementById("loginErrorText"),
        themeSwitcher: document.getElementById("themeSwitcher"),
        themeDarkBtn: document.getElementById("themeDarkBtn"),
        themeLightBtn: document.getElementById("themeLightBtn"),
        menuTabs: document.getElementById("menuTabs"),
        primaryOpsMenuCount: document.getElementById("primaryOpsMenuCount"),
        primaryOpsMenuList: document.getElementById("primaryOpsMenuList"),
        primaryOpsMenuSearch: document.getElementById("primaryOpsMenuSearch"),
        adminSidebar: document.getElementById("adminSidebar"),
        mobileNavToggle: document.getElementById("mobileNavToggle"),
        mobileNavBackdrop: document.getElementById("mobileNavBackdrop"),
        sidebarCloseBtn: document.getElementById("sidebarCloseBtn"),
        sidebarThemeBtn: document.getElementById("sidebarThemeBtn"),
        sidebarCommandCenterLink: document.getElementById("sidebarCommandCenterLink"),
        currentViewTitle: document.getElementById("currentViewTitle"),
        currentViewDescription: document.getElementById("currentViewDescription"),
        syncState: document.getElementById("syncState"),
        mainContent: document.getElementById("mainContent"),
        panels: Array.from(document.querySelectorAll(".panel")),
        logoutBtn: document.getElementById("logoutBtn"),
        currentAdminText: document.getElementById("currentAdminText"),
        authState: document.getElementById("authState"),
        usersTableBody: document.getElementById("usersTableBody"),
        usersPageInfo: document.getElementById("usersPageInfo"),
        usersTotalText: document.getElementById("usersTotalText"),
        usersSearchField: document.getElementById("usersSearchField"),
        usersSearchInput: document.getElementById("usersSearchInput"),
        usersStatusFilter: document.getElementById("usersStatusFilter"),
        usersPageSize: document.getElementById("usersPageSize"),
        usersSearchBtn: document.getElementById("usersSearchBtn"),
        usersClearBtn: document.getElementById("usersClearBtn"),
        loadUsersBtn: document.getElementById("loadUsersBtn"),
        prevUsersBtn: document.getElementById("prevUsersBtn"),
        nextUsersBtn: document.getElementById("nextUsersBtn"),
        refreshOverviewBtn: document.getElementById("refreshOverviewBtn"),
        openCommandCenterBtn: document.getElementById("openCommandCenterBtn"),
        commandCenterDashboard: document.getElementById("commandCenterDashboard"),
        commandCenterCountdown: document.getElementById("commandCenterCountdown"),
        commandCenterUpdatedAt: document.getElementById("commandCenterUpdatedAt"),
        commandCenterStatusText: document.getElementById("commandCenterStatusText"),
        overviewHint: document.getElementById("overviewHint"),
        apiVersionSelect: document.getElementById("apiVersionSelect"),
        apiMethodSelect: document.getElementById("apiMethodSelect"),
        apiEndpointInput: document.getElementById("apiEndpointInput"),
        apiBodyInput: document.getElementById("apiBodyInput"),
        sendApiBtn: document.getElementById("sendApiBtn"),
        presetThemeBtn: document.getElementById("presetThemeBtn"),
        presetLimitBtn: document.getElementById("presetLimitBtn"),
        apiResponseOutput: document.getElementById("apiResponseOutput"),
        opsModuleTag: document.getElementById("opsModuleTag"),
        opsModuleTitle: document.getElementById("opsModuleTitle"),
        opsModuleDesc: document.getElementById("opsModuleDesc"),
        opsWorkspace: document.getElementById("opsWorkspace"),
        opsRefreshBtn: document.getElementById("opsRefreshBtn"),
        opsOpenApiBtn: document.getElementById("opsOpenApiBtn"),
        actionDialog: document.getElementById("actionDialog"),
        actionDialogForm: document.getElementById("actionDialogForm"),
        actionDialogTitle: document.getElementById("actionDialogTitle"),
        actionDialogMessage: document.getElementById("actionDialogMessage"),
        actionDialogReasonField: document.getElementById("actionDialogReasonField"),
        actionDialogReason: document.getElementById("actionDialogReason"),
        actionDialogError: document.getElementById("actionDialogError"),
        actionDialogCancel: document.getElementById("actionDialogCancel"),
        actionDialogClose: document.getElementById("actionDialogClose"),
        actionDialogConfirm: document.getElementById("actionDialogConfirm"),
        toast: document.getElementById("toast")
      };

      const securePath = (settings.secure_path || "").replace(/^\/+|\/+$/g, "");
      const SHARED_LEGACY_TOKEN_KEY = "token";
      const SHARED_PROFILE_KEY = "me";
      const sharedAuthKeys = ["auth_data", "PORTAL_ACCESS_TOKEN", "Portal_access_token", "access_token"];
      const pendingMutationKeys = new Set();

      function normalizeBearer(token) {
        const clean = String(token || "").trim();
        if (!clean) return "";
        return clean.toLowerCase().startsWith("bearer ") ? clean : `Bearer ${clean}`;
      }

      function readSharedAuthToken() {
        for (const key of [...sharedAuthKeys, SHARED_LEGACY_TOKEN_KEY]) {
          const value = normalizeBearer(localStorage.getItem(key) || "");
          if (value) {
            return value;
          }
        }
        return "";
      }

      function persistSharedAuthToken(token) {
        const value = normalizeBearer(token);
        sharedAuthKeys.forEach((key) => {
          if (value) {
            localStorage.setItem(key, value);
          } else {
            localStorage.removeItem(key);
          }
        });
      }

      function persistSharedLegacyToken(token) {
        const value = String(token || "").trim();
        if (value) {
          localStorage.setItem(SHARED_LEGACY_TOKEN_KEY, value);
          return;
        }
        localStorage.removeItem(SHARED_LEGACY_TOKEN_KEY);
      }

      function persistSharedAdminProfile(profile = null) {
        if (!profile) {
          localStorage.removeItem("me");
          return;
        }

        localStorage.setItem("me", JSON.stringify({
          id: profile.id,
          email: profile.email,
          is_admin: !!profile.is_admin,
          is_super_admin: !!profile.is_super_admin,
          secure_path: securePath || profile.secure_path || null
        }));
      }

      function showToast(message, type = "info") {
        dom.toast.className = `toast ${type}`;
        dom.toast.textContent = message;
        dom.toast.classList.add("show");
        window.clearTimeout(showToast._timer);
        showToast._timer = window.setTimeout(() => dom.toast.classList.remove("show"), 2600);
      }

      function setAuthState(ok, label) {
        dom.authState.className = `badge ${ok ? "ok" : "warn"}`;
        dom.authState.textContent = label;
      }

      function updateToken(token, persist = true) {
        state.token = normalizeBearer(token);
        if (persist) {
          persistSharedAuthToken(state.token);
        }
      }

      function clearToken(clearSharedAuth = false) {
        clearOverviewRefresh();
        state.token = "";
        state.currentAdmin = "";
        state.users = [];
        state.usersLoaded = false;
        state.usersLoading = false;
        state.usersTotal = 0;
        state.usersRequestEpoch += 1;
        state.overviewLoaded = false;
        state.loadedOpsModules.clear();
        state.opsCache = createOpsCache();
        resetOpsDirtyState();
        state.navigationEpoch += 1;
        if (clearSharedAuth) {
          persistSharedAuthToken("");
          persistSharedLegacyToken("");
          persistSharedAdminProfile(null);
        }
        if (dom.currentAdminText) {
          dom.currentAdminText.textContent = "-";
        }
        if (dom.usersTableBody) {
          dom.usersTableBody.innerHTML = '<tr><td colspan="8" class="empty">登录后打开本页加载用户。</td></tr>';
        }
        setAuthState(false, "未认证");
      }

      function setLoginError(message = "") {
        dom.loginErrorText.textContent = message;
      }

      function showLoginView(message = "", options = {}) {
        const clearSharedAuth = Boolean(
          typeof options === "boolean" ? options : options?.clearSharedAuth
        );
        clearToken(clearSharedAuth);
        setLoginError(message);
        dom.adminLayout.classList.add("hidden");
        dom.loginView.classList.remove("hidden");
        prepareLoginSecurity().catch(() => {});
      }

      function showConsoleView() {
        setLoginError("");
        dom.loginView.classList.add("hidden");
        dom.adminLayout.classList.remove("hidden");
        setMobileNavigation(false);
      }

      function resetOpsDirtyState() {
        state.opsDirty = false;
        state.opsDirtyRevision += 1;
        state.opsDirtyScopes.clear();
        state.opsDirtyScopeRevisions = new WeakMap();
      }

      function markOpsDirty(target) {
        const scope = target?.closest?.(".ops-card") || dom.opsWorkspace;
        if (!scope) return;
        state.opsDirty = true;
        state.opsDirtyRevision += 1;
        state.opsDirtyScopes.add(scope);
        state.opsDirtyScopeRevisions.set(
          scope,
          Number(state.opsDirtyScopeRevisions.get(scope) || 0) + 1
        );
      }

      function headers(withJson = true) {
        const h = {
          Accept: "application/json"
        };
        if (withJson) {
          h["Content-Type"] = "application/json";
        }
        if (state.token) {
          h.Authorization = state.token;
        }
        return h;
      }

      function buildV2(endpoint) {
        const cleaned = String(endpoint || "").replace(/^\/+/, "");
        return `/api/v2/${securePath}/${cleaned}`;
      }

      function buildV1(endpoint) {
        const cleaned = String(endpoint || "").replace(/^\/+/, "");
        return `/api/v1/${cleaned}`;
      }

      async function syncAdminSession() {
        if (!state.token) {
          return null;
        }

        const body = await request({ method: "GET", url: buildV1("user/me") });
        const me = toData(body) || {};

        if (!me.is_admin || !me.is_super_admin) {
          const error = new Error("当前账号没有超级管理员权限");
          error.code = "not_super_admin";
          throw error;
        }

        state.currentAdmin = String(me.email || "");
        if (dom.currentAdminText) {
          dom.currentAdminText.textContent = state.currentAdmin || "-";
        }
        setAuthState(true, "已认证");
        persistSharedAdminProfile(me);
        return me;
      }

      async function request({ method = "GET", url, data, formData, readOnly = false, clearsDirty = false }) {
        const requestTab = state.activeTab;
        const requestModule = state.activeOpsModule;
        const dirtyMode = clearsDirty === "module" ? "module" : (clearsDirty ? "scope" : "");
        const dirtyRevision = state.opsDirtyRevision;
        const activeScope = document.activeElement?.closest?.(".ops-card") || null;
        const dirtyScope = activeScope || (state.opsDirtyScopes.size === 1
          ? state.opsDirtyScopes.values().next().value
          : null);
        const dirtyScopeRevision = dirtyScope
          ? Number(state.opsDirtyScopeRevisions.get(dirtyScope) || 0)
          : 0;
        const init = { method };
        if (formData) {
          init.headers = headers(false);
          delete init.headers["Content-Type"];
          init.body = formData;
        } else if (data !== undefined && method !== "GET") {
          init.headers = headers(true);
          init.body = JSON.stringify(data);
        } else {
          init.headers = headers(false);
        }

        const isMutation = !readOnly && method !== "GET";
        const mutationKey = isMutation ? `${method}:${url}` : "";
        if (mutationKey && pendingMutationKeys.has(mutationKey)) {
          throw new Error("相同操作正在执行，请等待完成");
        }
        const pendingButton = isMutation
          ? document.activeElement?.closest?.("#adminLayout button")
          : null;
        if (mutationKey) pendingMutationKeys.add(mutationKey);
        if (pendingButton) pendingButton.disabled = true;

        try {
          const res = await fetch(url, init);
          const contentType = res.headers.get("content-type") || "";
          const body = contentType.includes("application/json") ? await res.json() : await res.text();

          if (res.status === 401) {
            const error = new Error("未授权访问，请重新登录");
            error.status = 401;
            showLoginView(error.message, { clearSharedAuth: true });
            throw error;
          }

          if (!res.ok) {
            const message = extractError(body) || `请求失败 (${res.status})`;
            throw new Error(message);
          }

          if (isMutation && requestTab === "ops" && OPS_MODULE_MAP[requestModule]) {
            state.loadedOpsModules.delete(requestModule);
            if (dirtyMode && state.activeTab === "ops" && state.activeOpsModule === requestModule) {
              if (dirtyMode === "module" && state.opsDirtyRevision === dirtyRevision) {
                resetOpsDirtyState();
              } else if (
                dirtyMode === "scope"
                && dirtyScope
                && Number(state.opsDirtyScopeRevisions.get(dirtyScope) || 0) === dirtyScopeRevision
              ) {
                state.opsDirtyScopes.delete(dirtyScope);
                state.opsDirty = state.opsDirtyScopes.size > 0;
              }
            }
          }

          return body;
        } finally {
          if (mutationKey) pendingMutationKeys.delete(mutationKey);
          if (pendingButton?.isConnected) pendingButton.disabled = false;
        }
      }

      function extractError(body) {
        if (!body) return "";
        if (typeof body === "string") return body.slice(0, 160);
        return body.message || body.error || (body.data && body.data.message) || "";
      }

      function toData(body) {
        if (!body || typeof body !== "object") return body;
        if (Object.prototype.hasOwnProperty.call(body, "status")) {
          return body.data;
        }
        return body;
      }

      const LOGIN_COMM_CONFIG_URL = "/api/v1/guest/comm/config";
      const POW_CHALLENGE_URL = "/api/v1/passport/auth/pow-challenge";
      const TURNSTILE_SCRIPT_ID = "admin_turnstile_script";
      const RECAPTCHA_SCRIPT_ID = "admin_recaptcha_script";

      function isCaptchaEnabled(config) {
        return Number(config?.is_captcha || 0) === 1;
      }

      function getCaptchaProvider(config) {
        const type = String(config?.captcha_type || "recaptcha").toLowerCase();
        if (type === "turnstile") return "turnstile";
        if (type === "recaptcha-v3") return "recaptcha-v3";
        return "recaptcha";
      }

      function isPowEnabled(config) {
        return Number(config?.pow_enable || 0) === 1;
      }

      function setLoginPowHint(message = "", isError = false) {
        if (!dom.loginPowHint) return;
        const text = String(message || "").trim();
        dom.loginPowHint.classList.toggle("hidden", !text);
        dom.loginPowHint.classList.toggle("error", Boolean(isError));
        dom.loginPowHint.textContent = text;
      }

      function setLoginCaptchaHint(message = "") {
        if (!dom.loginCaptchaHint) return;
        dom.loginCaptchaHint.textContent = String(message || "").trim();
      }

      async function loadLoginCommConfig(force = false) {
        if (!force && state.loginCommConfig) return state.loginCommConfig;
        if (!force && state.loginCommConfigPromise) return state.loginCommConfigPromise;

        state.loginCommConfigPromise = request({ method: "GET", url: LOGIN_COMM_CONFIG_URL })
          .then((body) => {
            const cfg = toData(body);
            state.loginCommConfig = (cfg && typeof cfg === "object") ? cfg : {};
            return state.loginCommConfig;
          })
          .finally(() => {
            state.loginCommConfigPromise = null;
          });

        return state.loginCommConfigPromise;
      }

      function updateLoginSubmitAvailability() {
        const cfg = state.loginCommConfig || {};
        const captchaEnabled = isCaptchaEnabled(cfg);
        const powEnabled = isPowEnabled(cfg);
        const provider = state.loginCaptchaProvider || getCaptchaProvider(cfg);

        let captchaReady = true;
        if (captchaEnabled) {
          if (provider === "recaptcha-v3") {
            captchaReady = Boolean(state.loginCaptchaV3Ready);
          } else {
            captchaReady = Boolean(state.loginCaptchaToken);
          }
        }

        const powReady = !powEnabled || Boolean(state.loginPowProof);
        dom.loginSubmitBtn.disabled = !(captchaReady && powReady);
      }

      function waitFor(predicate, { timeoutMs = 8000, intervalMs = 80 } = {}) {
        return new Promise((resolve, reject) => {
          const start = Date.now();
          const timer = window.setInterval(() => {
            try {
              if (predicate()) {
                window.clearInterval(timer);
                resolve(true);
                return;
              }
              if (Date.now() - start >= timeoutMs) {
                window.clearInterval(timer);
                reject(new Error("脚本加载超时"));
              }
            } catch (err) {
              window.clearInterval(timer);
              reject(err);
            }
          }, intervalMs);
        });
      }

      function ensureScriptLoaded(id, src) {
        const desired = String(src || "").trim();
        if (!desired) return Promise.reject(new Error("脚本地址为空"));

        const existed = document.getElementById(id);
        if (existed) {
          const existedSrc = existed.getAttribute("src") || "";
          if (existedSrc === desired) {
            return Promise.resolve();
          }
          existed.remove();
        }

        return new Promise((resolve, reject) => {
          const script = document.createElement("script");
          script.id = id;
          script.src = desired;
          script.async = true;
          script.defer = true;
          script.onload = () => resolve();
          script.onerror = () => reject(new Error("脚本加载失败"));
          document.head.appendChild(script);
        });
      }

      async function ensureTurnstileLoaded() {
        await ensureScriptLoaded(TURNSTILE_SCRIPT_ID, "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit");
        await waitFor(() => window.turnstile && typeof window.turnstile.render === "function", { timeoutMs: 12000 });
      }

      function resolveRecaptchaScriptSrc(provider, siteKey) {
        const p = String(provider || "recaptcha").toLowerCase();
        if (p === "recaptcha-v3") {
          const key = String(siteKey || "").trim();
          return `https://www.recaptcha.net/recaptcha/api.js?render=${encodeURIComponent(key)}`;
        }
        return "https://www.recaptcha.net/recaptcha/api.js?render=explicit";
      }

      async function ensureRecaptchaLoaded(provider, siteKey) {
        const src = resolveRecaptchaScriptSrc(provider, siteKey);
        await ensureScriptLoaded(RECAPTCHA_SCRIPT_ID, src);
        if (String(provider || "").toLowerCase() === "recaptcha-v3") {
          await waitFor(() => window.grecaptcha && typeof window.grecaptcha.execute === "function" && typeof window.grecaptcha.ready === "function", { timeoutMs: 12000 });
          return;
        }
        await waitFor(() => window.grecaptcha && typeof window.grecaptcha.render === "function", { timeoutMs: 12000 });
      }

      function resetLoginCaptchaWidget() {
        const provider = String(state.loginCaptchaProvider || "").toLowerCase();
        const widgetId = state.loginCaptchaWidgetId;

        state.loginCaptchaToken = "";
        state.loginCaptchaWidgetId = null;
        state.loginCaptchaV3Ready = false;

        if (provider === "turnstile" && window.turnstile && widgetId != null) {
          try { window.turnstile.reset(widgetId); } catch (_) {}
        }
        if (provider === "recaptcha" && window.grecaptcha && widgetId != null) {
          try { window.grecaptcha.reset(widgetId); } catch (_) {}
        }

        if (dom.loginCaptchaWidget) {
          dom.loginCaptchaWidget.innerHTML = "";
        }
      }

      async function prewarmLoginCaptcha() {
        const cfg = await loadLoginCommConfig().catch(() => ({}));
        const enabled = isCaptchaEnabled(cfg);
        if (!dom.loginCaptchaField) return;

        resetLoginCaptchaWidget();
        dom.loginCaptchaField.classList.toggle("hidden", !enabled);

        if (!enabled) {
          state.loginCaptchaProvider = "";
          setLoginCaptchaHint("");
          updateLoginSubmitAvailability();
          return;
        }

        const provider = getCaptchaProvider(cfg);
        state.loginCaptchaProvider = provider;

        if (provider === "turnstile") {
          const siteKey = String(cfg.turnstile_site_key || "").trim();
          if (!siteKey) {
            setLoginCaptchaHint("Turnstile 站点密钥未配置");
            updateLoginSubmitAvailability();
            return;
          }
          setLoginCaptchaHint("请完成 Turnstile 人机验证后登录。");
          await ensureTurnstileLoaded();
          const wid = window.turnstile.render(dom.loginCaptchaWidget, {
            sitekey: siteKey,
            callback: (token) => {
              state.loginCaptchaToken = String(token || "").trim();
              updateLoginSubmitAvailability();
            },
            "expired-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            },
            "error-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            }
          });
          state.loginCaptchaWidgetId = wid;
          updateLoginSubmitAvailability();
          return;
        }

        if (provider === "recaptcha") {
          const siteKey = String(cfg.recaptcha_site_key || "").trim();
          if (!siteKey) {
            setLoginCaptchaHint("reCAPTCHA 站点密钥未配置");
            updateLoginSubmitAvailability();
            return;
          }
          setLoginCaptchaHint("请完成 reCAPTCHA 人机验证后登录。");
          await ensureRecaptchaLoaded("recaptcha", siteKey);
          const wid = window.grecaptcha.render(dom.loginCaptchaWidget, {
            sitekey: siteKey,
            callback: (token) => {
              state.loginCaptchaToken = String(token || "").trim();
              updateLoginSubmitAvailability();
            },
            "expired-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            },
            "error-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            }
          });
          state.loginCaptchaWidgetId = wid;
          updateLoginSubmitAvailability();
          return;
        }

        if (provider === "recaptcha-v3") {
          const siteKey = String(cfg.recaptcha_v3_site_key || "").trim();
          if (!siteKey) {
            setLoginCaptchaHint("reCAPTCHA v3 站点密钥未配置");
            updateLoginSubmitAvailability();
            return;
          }
          setLoginCaptchaHint("当前启用 reCAPTCHA v3（无感验证），点击登录将自动完成校验。");
          await ensureRecaptchaLoaded("recaptcha-v3", siteKey);
          state.loginCaptchaV3Ready = true;
          updateLoginSubmitAvailability();
          return;
        }

        setLoginCaptchaHint("未知验证码服务类型");
        updateLoginSubmitAvailability();
      }

      async function sha256Hex(message) {
        if (!window.crypto || !window.crypto.subtle || !window.TextEncoder) {
          throw new Error("浏览器不支持 WebCrypto，无法计算 PoW");
        }
        const data = new TextEncoder().encode(String(message || ""));
        const digest = await window.crypto.subtle.digest("SHA-256", data);
        return Array.from(new Uint8Array(digest))
          .map((b) => b.toString(16).padStart(2, "0"))
          .join("");
      }

      function yieldControl() {
        return new Promise((resolve) => window.setTimeout(resolve, 0));
      }

      async function loadPowChallenge() {
        const res = await fetch(POW_CHALLENGE_URL, { credentials: "same-origin" });
        const text = await res.text();
        let json = null;
        try { json = text ? JSON.parse(text) : null; } catch (_) { json = null; }
        if (!res.ok) {
          throw new Error(json?.message || json?.error || `HTTP ${res.status}`);
        }
        return json?.data || null;
      }

      async function computePowProof(challenge) {
        if (!challenge || typeof challenge !== "object") return null;
        const difficulty = Number(challenge.difficulty || 4);
        const prefix = "0".repeat(Math.max(1, difficulty));
        let nonce = 0;

        while (true) {
          const input = `${challenge.seed}|${challenge.base}|${challenge.token}|${nonce}`;
          const hash = await sha256Hex(input);
          if (hash.startsWith(prefix)) {
            return {
              id: challenge.challenge_id,
              nonce: String(nonce),
              hash,
              token: challenge.token,
              expires_at: challenge.expires_at,
            };
          }
          nonce += 1;
          if (nonce % 250 === 0) {
            await yieldControl();
          }
        }
      }

      async function ensureLoginPowProof() {
        const cfg = state.loginCommConfig || await loadLoginCommConfig().catch(() => ({}));
        if (!isPowEnabled(cfg)) {
          state.loginPowProof = null;
          setLoginPowHint("");
          updateLoginSubmitAvailability();
          return null;
        }

        const current = state.loginPowProof;
        const now = Math.floor(Date.now() / 1000);
        if (current?.expires_at && Number(current.expires_at) > now + 5) {
          return current;
        }

        if (state.loginPowPromise) {
          return state.loginPowPromise;
        }

        setLoginPowHint(`正在准备防刷验证，当前难度 ${Number(cfg?.pow_effective_difficulty || cfg?.pow_difficulty || 4)}，请稍候...`);
        updateLoginSubmitAvailability();

        state.loginPowPromise = (async () => {
          const challenge = await loadPowChallenge();
          const proof = await computePowProof(challenge);
          state.loginPowProof = proof;
          setLoginPowHint(`防刷验证已准备完成，当前难度 ${Number(cfg?.pow_effective_difficulty || cfg?.pow_difficulty || 4)}。`);
          return proof;
        })()
          .catch((err) => {
            state.loginPowProof = null;
            setLoginPowHint(err.message || "防刷验证初始化失败", true);
            throw err;
          })
          .finally(() => {
            state.loginPowPromise = null;
            updateLoginSubmitAvailability();
          });

        return state.loginPowPromise;
      }

      async function prepareLoginSecurity() {
        await loadLoginCommConfig().catch(() => ({}));
        await prewarmLoginCaptcha().catch((err) => {
          setLoginCaptchaHint(err.message || "验证码加载失败");
        });
        await ensureLoginPowProof().catch(() => {});
        updateLoginSubmitAvailability();
      }

      function money(v) {
        const n = Number(v || 0);
        return Number.isFinite(n) ? n.toLocaleString("zh-CN", { maximumFractionDigits: 2 }) : "0";
      }

      function escapeHtml(v) {
        return String(v == null ? "" : v)
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;")
          .replace(/"/g, "&quot;")
          .replace(/'/g, "&#39;");
      }

      const OPS_NAV_GROUPS = [
        { key: "risk", name: "用户与风险" },
        { key: "business", name: "产品与资金" },
        { key: "support", name: "内容与支持" },
        { key: "access", name: "平台与接入" },
        { key: "system", name: "系统维护" }
      ];

      const OPS_GROUP_MAP = OPS_NAV_GROUPS.reduce((acc, item) => {
        acc[item.key] = item;
        return acc;
      }, {});

      const OPS_MODULES = [
        { key: "riskreview", name: "风险审查", desc: "共享 IP 风险、预审查与封禁记录", group: "risk", tag: "RISK", render: renderOpsRiskReview, bind: bindOpsRiskReview },
        { key: "traffic", name: "流量重置", desc: "用户流量重置与操作记录", group: "risk", tag: "RESET", render: renderOpsTraffic, bind: bindOpsTraffic },
        { key: "plans", name: "套餐管理", desc: "价格、展示与上下架", group: "business", tag: "PLAN", render: renderOpsPlans, bind: bindOpsPlans },
        { key: "payments", name: "支付渠道", desc: "支付方式与启停状态", group: "business", tag: "PAY", render: renderOpsPayments, bind: bindOpsPayments },
        { key: "coupons", name: "优惠券", desc: "生成、展示与删除", group: "business", tag: "COUPON", render: renderOpsCoupons, bind: bindOpsCoupons },
        { key: "giftcards", name: "礼品卡", desc: "模板、兑换码与统计", group: "business", tag: "GIFT", render: renderOpsGiftCards, bind: bindOpsGiftCards },
        { key: "tickets", name: "工单管理", desc: "查看、回复与关闭", group: "support", tag: "TICKET", render: renderOpsTickets, bind: bindOpsTickets },
        { key: "notices", name: "公告管理", desc: "发布与显示状态", group: "support", tag: "NOTICE", render: renderOpsNotices, bind: bindOpsNotices },
        { key: "security", name: "登录与安全", desc: "登录保护、验证码与后台入口", group: "access", tag: "ACCESS", render: renderOpsSecurity, bind: bindOpsSecurity },
        { key: "oauth", name: "第三方登录", desc: "Linux DO OAuth 与回调", group: "access", tag: "OAUTH", render: renderOpsOAuth, bind: bindOpsOAuth },
        { key: "site", name: "站点与订阅", desc: "站点、主题与订阅地址", group: "access", tag: "SITE", render: renderOpsSite, bind: bindOpsSite },
        { key: "telegram", name: "通知设置", desc: "Telegram Bot 与事件通知", group: "access", tag: "NOTICE", render: renderOpsTelegram, bind: bindOpsTelegram },
        { key: "system", name: "系统健康", desc: "运行状态、队列与日志", group: "system", tag: "SYSTEM", render: renderOpsSystem, bind: bindOpsSystem },
        { key: "plugins", name: "扩展功能", desc: "安装、启停、升级与配置", group: "system", tag: "PLUGIN", render: renderOpsPlugins, bind: bindOpsPlugins }
      ];

      const OPS_MODULE_MAP = OPS_MODULES.reduce((acc, item) => {
        acc[item.key] = item;
        return acc;
      }, {});

      const MODULE_CONTROLLERS = Object.freeze(OPS_MODULES.reduce((acc, module) => {
        acc[module.key] = Object.freeze({
          load: (force = false) => loadOpsModuleData(module.key, force),
          render: module.render,
          bind: module.bind
        });
        return acc;
      }, {}));

      const ADMIN_THEME_STORAGE_KEY = "notxboard_admin_console_theme";

      function normalizeAdminTheme(value) {
        return String(value || "").toLowerCase() === "dark" ? "dark" : "light";
      }

      function syncThemeButtons() {
        if (dom.themeDarkBtn) {
          dom.themeDarkBtn.classList.toggle("active", state.adminTheme === "dark");
          dom.themeDarkBtn.setAttribute("aria-pressed", state.adminTheme === "dark" ? "true" : "false");
        }
        if (dom.themeLightBtn) {
          dom.themeLightBtn.classList.toggle("active", state.adminTheme === "light");
          dom.themeLightBtn.setAttribute("aria-pressed", state.adminTheme === "light" ? "true" : "false");
        }
        if (dom.sidebarThemeBtn) {
          const nextTheme = state.adminTheme === "dark" ? "浅色" : "深色";
          dom.sidebarThemeBtn.setAttribute("aria-label", `切换到${nextTheme}主题`);
          dom.sidebarThemeBtn.title = `切换到${nextTheme}主题`;
        }
      }

      function moduleMatchesSearch(module, keyword = "") {
        if (!module) return false;
        const query = String(keyword || "").trim().toLowerCase();
        if (!query) return true;
        const groupName = OPS_GROUP_MAP[module.group]?.name || "";
        return [module.name, module.desc, module.tag, groupName]
          .filter(Boolean)
          .some((text) => String(text).toLowerCase().includes(query));
      }

      function applyAdminTheme(theme, persist = true) {
        const nextTheme = normalizeAdminTheme(theme);
        state.adminTheme = nextTheme;
        document.body.setAttribute("data-admin-theme", nextTheme);
        syncThemeButtons();
        if (persist) {
          localStorage.setItem(ADMIN_THEME_STORAGE_KEY, nextTheme);
        }
      }

      function renderPrimaryOpsMenu(keyword = "") {
        if (!dom.primaryOpsMenuList) return;
        const query = String(keyword || "").trim().toLowerCase();
        let totalVisible = 0;
        const sections = OPS_NAV_GROUPS.map((group) => {
          const modules = OPS_MODULES.filter((module) => {
            if (module.group !== group.key) return false;
            return moduleMatchesSearch(module, query);
          });
          if (!modules.length) return "";
          totalVisible += modules.length;
          const groupIsActive = state.activeTab === "ops"
            && modules.some((module) => module.key === state.activeOpsModule);
          return `
            <details class="menu-subgroup" ${query || groupIsActive ? "open" : ""}>
              <summary>
                <span class="menu-subgroup-title">${escapeHtml(group.name)}</span>
                <span class="menu-subgroup-count">${modules.length}</span>
              </summary>
              <div class="menu-subgroup-list">
                ${modules.map((module) => `
                  <button type="button" class="menu-btn menu-btn-compact ${state.activeTab === "ops" && state.activeOpsModule === module.key ? "active" : ""}" data-tab="ops" data-ops-module="${module.key}">
                    <span class="menu-marker" aria-hidden="true"></span>
                    <span class="menu-btn-title">${escapeHtml(module.name)}</span>
                    <span class="menu-btn-index">${escapeHtml(module.tag || "MOD")}</span>
                  </button>
                `).join("")}
              </div>
            </details>
          `;
        }).filter(Boolean);

        if (dom.primaryOpsMenuCount) {
          dom.primaryOpsMenuCount.textContent = query
            ? `${totalVisible} / ${OPS_MODULES.length}`
            : `${OPS_MODULES.length}`;
        }

        dom.primaryOpsMenuList.innerHTML = sections.length
          ? sections.join("")
          : '<p class="empty">没有匹配的模块，请调整检索关键字。</p>';
      }

      function setActiveMenuButton(tab, moduleKey = "") {
        const nextTab = String(tab || "overview");
        const nextModule = String(moduleKey || "");
        document.querySelectorAll(".menu-btn").forEach((btn) => {
          const btnTab = btn.dataset.tab || "";
          const btnModule = btn.dataset.opsModule || "";
          const active = nextTab === "ops"
            ? (btnTab === "ops" && btnModule === nextModule)
            : (btnTab === nextTab && !btnModule);
          btn.classList.toggle("active", active);
          if (active) {
            btn.setAttribute("aria-current", "page");
          } else {
            btn.removeAttribute("aria-current");
          }
        });
      }

      const VIEW_META = Object.freeze({
        overview: { title: "工作台", description: "今日状态与高频操作" },
        users: { title: "用户与封禁", description: "搜索用户并执行账号处置" },
        api: { title: "高级请求工具", description: "使用当前超级管理员会话调试接口" }
      });

      function viewRoute(tab, moduleKey = "") {
        return tab === "ops" && OPS_MODULE_MAP[moduleKey]
          ? `#ops/${encodeURIComponent(moduleKey)}`
          : `#${VIEW_META[tab] ? tab : "overview"}`;
      }

      function parseViewRoute(rawHash = window.location.hash) {
        const route = String(rawHash || "").replace(/^#/, "").split("/").filter(Boolean);
        if (route[0] === "ops" && OPS_MODULE_MAP[route[1]]) {
          return { tab: "ops", moduleKey: route[1] };
        }
        if (VIEW_META[route[0]]) {
          return { tab: route[0], moduleKey: "" };
        }
        return { tab: "overview", moduleKey: "" };
      }

      function syncViewRoute(tab, moduleKey = "", replace = false) {
        const nextHash = viewRoute(tab, moduleKey);
        if (window.location.hash === nextHash) return;
        const nextUrl = `${window.location.pathname}${window.location.search}${nextHash}`;
        window.history[replace ? "replaceState" : "pushState"](
          { tab, moduleKey },
          "",
          nextUrl
        );
      }

      function updateViewHeading(tab, moduleKey = "") {
        const module = tab === "ops" ? OPS_MODULE_MAP[moduleKey] : null;
        const meta = module || VIEW_META[tab] || VIEW_META.overview;
        if (dom.currentViewTitle) dom.currentViewTitle.textContent = meta.name || meta.title;
        if (dom.currentViewDescription) dom.currentViewDescription.textContent = meta.desc || meta.description;
        if (dom.syncState) dom.syncState.hidden = tab !== "overview";
        if (dom.refreshOverviewBtn) dom.refreshOverviewBtn.disabled = tab === "api";
        document.title = `${meta.name || meta.title} - ${settings.title || "超级管理员"}`;
      }

      function setMobileNavigation(open) {
        const mobile = window.matchMedia("(max-width: 900px)").matches;
        const nextOpen = mobile && Boolean(open);
        document.body.classList.toggle("nav-open", nextOpen);
        if (dom.mobileNavToggle) {
          dom.mobileNavToggle.setAttribute("aria-expanded", nextOpen ? "true" : "false");
        }
        if (dom.mobileNavBackdrop) {
          dom.mobileNavBackdrop.tabIndex = nextOpen ? 0 : -1;
        }
        if (dom.mainContent) dom.mainContent.inert = nextOpen;
        if (dom.adminSidebar) {
          dom.adminSidebar.inert = mobile && !nextOpen;
          dom.adminSidebar.setAttribute("aria-hidden", mobile && !nextOpen ? "true" : "false");
        }
      }

      async function loadActiveView(options = {}) {
        if (!state.token) return;
        const force = Boolean(options.force);
        if (state.activeTab === "overview") {
          if (!state.overviewLoaded || force) {
            return (await loadOverview(force)) !== false;
          } else {
            scheduleOverviewRefresh(30);
          }
          return true;
        }
        clearOverviewRefresh();
        if (state.activeTab === "users") {
          if (!state.usersLoaded || force) return (await loadUsers()) !== false;
          return true;
        }
        if (state.activeTab === "ops") {
          return openOpsModule(state.activeOpsModule || "security", { force });
        }
        return true;
      }

      async function activateView(tab, moduleKey = "", options = {}) {
        const mobileNavWasOpen = document.body.classList.contains("nav-open");
        const nextTab = tab === "ops" || VIEW_META[tab] ? tab : "overview";
        const nextModule = nextTab === "ops" && OPS_MODULE_MAP[moduleKey]
          ? moduleKey
          : state.activeOpsModule || "security";
        const sameView = nextTab === state.activeTab
          && (nextTab !== "ops" || nextModule === state.activeOpsModule);
        if (sameView && options.load !== false && !options.force) {
          setMobileNavigation(false);
          if (options.syncRoute !== false) syncViewRoute(nextTab, nextModule, false);
          return true;
        }
        const discardsOpsChanges = state.opsDirty
          && state.activeTab === "ops"
          && (!sameView || Boolean(options.force));
        if (discardsOpsChanges && !options.discardChanges) {
          const decision = await requestActionConfirmation({
            title: "放弃未保存修改",
            message: "当前模块有未保存的修改。切换后这些修改会丢失。",
            confirmLabel: "放弃修改"
          });
          if (!decision.confirmed) return false;
        }
        if (discardsOpsChanges) resetOpsDirtyState();
        state.activeTab = nextTab;
        state.navigationEpoch += 1;
        if (nextTab === "ops") state.activeOpsModule = nextModule;

        renderPrimaryOpsMenu(readValue("primaryOpsMenuSearch", ""));
        setActiveMenuButton(nextTab, nextTab === "ops" ? nextModule : "");
        dom.panels.forEach((panel) => {
          panel.classList.toggle("active", panel.dataset.panel === nextTab);
        });
        updateViewHeading(nextTab, nextModule);
        setMobileNavigation(false);
        if (mobileNavWasOpen && options.focusContent !== false) {
          window.setTimeout(() => dom.mainContent?.focus(), 0);
        }

        if (options.syncRoute !== false) {
          syncViewRoute(nextTab, nextModule, Boolean(options.replaceRoute));
        }
        if (options.load !== false) {
          await loadActiveView({ force: Boolean(options.force) });
        }
      }

      function settleActionDialog(result) {
        const resolve = state.actionDialogResolve;
        state.actionDialogResolve = null;
        if (dom.actionDialog?.open && typeof dom.actionDialog.close === "function") {
          dom.actionDialog.close();
        } else {
          dom.actionDialog?.removeAttribute("open");
        }
        if (resolve) resolve(result);
      }

      function requestActionConfirmation(options = {}) {
        if (!dom.actionDialog) {
          return Promise.resolve({ confirmed: false, reason: "" });
        }
        if (state.actionDialogResolve) {
          settleActionDialog({ confirmed: false, reason: "" });
        }

        const requireReason = Boolean(options.requireReason);
        dom.actionDialogTitle.textContent = options.title || "确认操作";
        dom.actionDialogMessage.textContent = options.message || "此操作将立即生效。";
        dom.actionDialogReasonField.classList.toggle("hidden", !options.showReason && !requireReason);
        dom.actionDialogReason.value = options.defaultReason || "";
        dom.actionDialogReason.placeholder = options.reasonPlaceholder || "";
        dom.actionDialogReason.dataset.required = requireReason ? "true" : "false";
        dom.actionDialogReason.required = requireReason;
        dom.actionDialogReason.setAttribute("aria-invalid", "false");
        dom.actionDialogError.textContent = "";
        dom.actionDialogConfirm.textContent = options.confirmLabel || "确认";
        dom.actionDialogConfirm.className = `btn ${options.tone === "primary" ? "primary" : "danger"}`;

        const promise = new Promise((resolve) => {
          state.actionDialogResolve = resolve;
        });
        if (typeof dom.actionDialog.showModal === "function") {
          dom.actionDialog.showModal();
        } else {
          dom.actionDialog.setAttribute("open", "");
        }
        window.setTimeout(() => {
          (requireReason || options.showReason ? dom.actionDialogReason : dom.actionDialogConfirm)?.focus();
        }, 0);
        return promise;
      }

      function isOn(v) {
        return v === true || Number(v) === 1 || String(v) === "1";
      }

      function yesNo(v) {
        return isOn(v) ? "开启" : "关闭";
      }

      function readValue(id, fallback = "") {
        const el = document.getElementById(id);
        if (!el) return fallback;
        return el.value != null ? el.value : fallback;
      }

      function readInt(id, fallback = 0, nullable = false) {
        const raw = String(readValue(id, "")).trim();
        if (!raw) return nullable ? null : fallback;
        const n = Number(raw);
        if (!Number.isFinite(n)) return nullable ? null : fallback;
        return Math.trunc(n);
      }

      function readFloat(id, fallback = 0, nullable = false) {
        const raw = String(readValue(id, "")).trim();
        if (!raw) return nullable ? null : fallback;
        const n = Number(raw);
        return Number.isFinite(n) ? n : (nullable ? null : fallback);
      }

      function readBoolControl(id) {
        const el = document.getElementById(id);
        if (!el) return false;
        return el.type === "checkbox" ? el.checked : isOn(readValue(id, "0"));
      }

      function setInputValue(id, value) {
        const el = document.getElementById(id);
        if (!el) return;
        if (el.type === "checkbox") {
          el.checked = isOn(value);
          return;
        }
        el.value = value == null ? "" : String(value);
      }

      function parseJsonText(raw, fallback = {}) {
        const text = String(raw || "").trim();
        if (!text) return fallback;
        return JSON.parse(text);
      }

      function parseJsonInput(id, fallback = {}) {
        return parseJsonText(readValue(id, ""), fallback);
      }

      function splitLines(raw) {
        return String(raw || "")
          .split(/\r?\n|,/)
          .map((s) => s.trim())
          .filter(Boolean);
      }

      function splitCsv(raw) {
        return String(raw || "")
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean);
      }

      function toUnixFromInput(raw) {
        if (!raw) return 0;
        const ms = new Date(raw).getTime();
        return Number.isFinite(ms) ? Math.floor(ms / 1000) : 0;
      }

      function toInputDatetime(ts) {
        const n = Number(ts || 0);
        if (!Number.isFinite(n) || n <= 0) return "";
        const d = new Date(n * 1000);
        const pad = (v) => String(v).padStart(2, "0");
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
      }

      function formatDateTime(ts) {
        const n = Number(ts || 0);
        if (!Number.isFinite(n) || n <= 0) return "-";
        return new Date(n * 1000).toLocaleString("zh-CN", { hour12: false });
      }

      function formatCompactNumber(value) {
        const num = Number(value);
        if (!Number.isFinite(num)) return "0";
        if (typeof Intl !== "undefined" && Intl.NumberFormat) {
          return new Intl.NumberFormat("zh-CN", { notation: "compact", maximumFractionDigits: 1 }).format(num);
        }
        return String(Math.round(num));
      }

      function formatAnyTimestamp(rawValue, fallback = "-") {
        if (rawValue === null || rawValue === undefined || rawValue === "") return fallback;
        if (typeof rawValue === "number" && Number.isFinite(rawValue)) return formatDateTime(rawValue);

        const text = String(rawValue).trim();
        if (!text) return fallback;

        const asInt = Number(text);
        if (Number.isFinite(asInt) && /^\d+$/.test(text)) {
          return formatDateTime(asInt);
        }

        const parsed = new Date(text);
        if (!Number.isNaN(parsed.getTime())) {
          return parsed.toLocaleString("zh-CN", { hour12: false });
        }

        return text;
      }

      function buildOverviewLanding(meta = {}) {
        const monitorPath = meta.securePath ? `/${meta.securePath}/command-center` : "/command-center";
        const stats = meta.stats || {};
        const pendingTickets = Number(stats.ticket_pending_total || 0);
        return `
          <section class="overview-grid">
            <article class="command-panel">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">待办与风险</div>
                  <h3>需要处理</h3>
                </div>
                ${buildCommandCenterStatusBadge(pendingTickets > 0 ? `${formatCompactNumber(pendingTickets)} 个待处理工单` : "暂无待处理工单", pendingTickets > 0 ? "warn" : "ok")}
              </div>
              <div class="overview-quick-grid">
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="tickets">
                  <span>工单</span>
                  <strong>${pendingTickets > 0 ? `处理 ${formatCompactNumber(pendingTickets)} 个待办` : "查看工单队列"}</strong>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="riskreview">
                  <span>风险</span>
                  <strong>风险审查与封禁流水</strong>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="system">
                  <span>系统</span>
                  <strong>队列、日志与失败任务</strong>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="users">
                  <span>用户</span>
                  <strong>搜索与账号处置</strong>
                </button>
              </div>
            </article>

            <article class="command-panel">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">常用入口</div>
                  <h3>业务维护</h3>
                </div>
                ${buildCommandCenterStatusBadge(monitorPath, "cyan")}
              </div>
              <div class="overview-quick-grid">
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="plans">
                  <span>产品</span>
                  <strong>套餐与价格</strong>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="payments">
                  <span>资金</span>
                  <strong>支付渠道</strong>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="notices">
                  <span>内容</span>
                  <strong>发布公告</strong>
                </button>
                <button type="button" class="overview-quick-card" data-overview-open="command-center">
                  <span>监控</span>
                  <strong>打开实时监控</strong>
                </button>
              </div>
            </article>
          </section>
        `;
      }

      function buildCommandCenterStatusBadge(label, tone = "neutral") {
        return `<span class="command-badge ${escapeHtml(tone)}">${escapeHtml(label)}</span>`;
      }

      function clearOverviewRefresh() {
        if (state.overviewRefreshTimer) {
          window.clearTimeout(state.overviewRefreshTimer);
          state.overviewRefreshTimer = null;
        }
        if (state.overviewCountdownTimer) {
          window.clearInterval(state.overviewCountdownTimer);
          state.overviewCountdownTimer = null;
        }
        state.overviewNextRefreshIn = 0;
      }

      function setOverviewStatus(text, tone = "neutral") {
        if (!dom.commandCenterStatusText) return;
        dom.commandCenterStatusText.textContent = text;
        dom.commandCenterStatusText.setAttribute("data-tone", tone);
      }

      function updateOverviewCountdown() {
        if (!dom.commandCenterCountdown) return;
        const remaining = Math.max(0, Math.floor(state.overviewNextRefreshIn || 0));
        const minutes = Math.floor(remaining / 60);
        const seconds = remaining % 60;
        dom.commandCenterCountdown.textContent = `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
      }

      function scheduleOverviewRefresh(seconds) {
        clearOverviewRefresh();
        if (!state.token || state.activeTab !== "overview") {
          updateOverviewCountdown();
          return;
        }
        state.overviewRefreshIntervalSeconds = Math.max(10, Number(seconds) || 20);
        state.overviewNextRefreshIn = state.overviewRefreshIntervalSeconds;
        updateOverviewCountdown();
        state.overviewCountdownTimer = window.setInterval(() => {
          if (!state.token) {
            clearOverviewRefresh();
            return;
          }
          state.overviewNextRefreshIn = Math.max(0, state.overviewNextRefreshIn - 1);
          updateOverviewCountdown();
        }, 1000);
        state.overviewRefreshTimer = window.setTimeout(() => {
          if (state.activeTab !== "overview") return;
          loadOverview(false).catch((err) => {
            showToast(err.message || "监控刷新失败", "error");
          });
        }, state.overviewRefreshIntervalSeconds * 1000);
      }

      function randomSecurePath() {
        return `admin-${Math.random().toString(36).slice(2, 10)}${Date.now().toString(36).slice(-4)}`;
      }

      function normalizeList(body) {
        const data = toData(body);
        if (Array.isArray(data)) return data;
        if (data && Array.isArray(data.data)) return data.data;
        if (body && Array.isArray(body.data)) return body.data;
        return [];
      }

      function normalizeObject(body) {
        const data = toData(body);
        if (data && typeof data === "object") return data;
        if (body && typeof body === "object") return body;
        return {};
      }

      function normalizeConfigValue(v, fallback = "") {
        return v == null ? fallback : v;
      }

      function opsBoolField(id, label, value, span = 3) {
        return `
          <div class="field toggle-field span-${span}">
            <label class="toggle-control" for="${id}">
              <input id="${id}" type="checkbox" role="switch" value="1" ${isOn(value) ? "checked" : ""} />
              <span class="toggle-track" aria-hidden="true"><span></span></span>
              <span class="toggle-copy">
                <span>${escapeHtml(label)}</span>
                <small class="toggle-state" aria-hidden="true"></small>
              </span>
            </label>
          </div>
        `;
      }

      function setPreOutput(id, value) {
        const el = document.getElementById(id);
        if (!el) return;
        el.textContent = typeof value === "string" ? value : JSON.stringify(value, null, 2);
      }

      function enhanceWorkspaceTables(root, label) {
        if (!root) return;
        root.querySelectorAll("table").forEach((table, index) => {
          if (!table.querySelector("caption")) {
            const caption = document.createElement("caption");
            caption.className = "sr-only";
            caption.textContent = `${label || "当前模块"}数据表${index + 1}`;
            table.prepend(caption);
          }
          table.querySelectorAll("thead th").forEach((header) => {
            if (!header.hasAttribute("scope")) header.setAttribute("scope", "col");
          });
          const lastHeader = table.querySelector("thead th:last-child");
          table.classList.toggle(
            "has-sticky-actions",
            Boolean(lastHeader && lastHeader.textContent.trim() === "操作")
          );
        });
      }

      function setOpsModuleHeader(moduleKey) {
        const module = OPS_MODULE_MAP[moduleKey] || OPS_MODULES[0];
        dom.opsModuleTitle.textContent = module.name;
        dom.opsModuleDesc.textContent = module.desc;
        if (dom.opsModuleTag) {
          const groupLabel = OPS_GROUP_MAP[module.group]?.name || "模块";
          dom.opsModuleTag.innerHTML = `<strong>${escapeHtml(module.tag || "MODULE")}</strong> ${escapeHtml(groupLabel)}`;
        }
      }

      async function ensureOpsConfig(force = false) {
        if (!force && state.opsCache.config) {
          return state.opsCache.config;
        }
        const body = await request({ method: "GET", url: buildV2("config/fetch") });
        state.opsCache.config = normalizeObject(body);
        return state.opsCache.config;
      }

      async function loadOpsModuleData(moduleKey, force = false) {
        if (!force && state.loadedOpsModules.has(moduleKey)) return;

        if (moduleKey === "security" || moduleKey === "oauth" || moduleKey === "site" || moduleKey === "telegram") {
          await ensureOpsConfig(force);
          return;
        }

        if (moduleKey === "riskreview") {
          if (!force && state.opsCache.config && state.opsCache.riskReviewPagination && state.opsCache.banRecordPagination) return;
          const riskReviewPage = Math.max(1, Number(state.opsCache.riskReviewPage || 1));
          const banRecordPage = Math.max(1, Number(state.opsCache.banRecordPage || 1));
          const [configBody, reviewsBody, banRecordsBody] = await Promise.all([
            ensureOpsConfig(force),
            request({ method: "GET", url: `${buildV2("risk-review/fetch")}?current=${riskReviewPage}&pageSize=20` }),
            request({ method: "GET", url: `${buildV2("user/ban-records")}?current=${banRecordPage}&pageSize=20` }),
          ]);
          state.opsCache.config = normalizeObject(configBody);
          const reviewsObj = normalizeObject(reviewsBody);
          const banRecordsObj = normalizeObject(banRecordsBody);
          state.opsCache.riskReviews = Array.isArray(reviewsObj.data) ? reviewsObj.data : [];
          state.opsCache.riskReviewPagination = {
            current_page: Number(reviewsObj.current_page || 1),
            last_page: Number(reviewsObj.last_page || 1),
            total: Number(reviewsObj.total || 0),
          };
          state.opsCache.banRecords = Array.isArray(banRecordsObj.data) ? banRecordsObj.data : [];
          state.opsCache.banRecordPagination = {
            current_page: Number(banRecordsObj.current_page || 1),
            last_page: Number(banRecordsObj.last_page || 1),
            total: Number(banRecordsObj.total || 0),
          };
          state.opsCache.riskReviewPage = state.opsCache.riskReviewPagination.current_page || 1;
          state.opsCache.banRecordPage = state.opsCache.banRecordPagination.current_page || 1;
          return;
        }

        if (moduleKey === "plans") {
          if (!force && state.opsCache.plans.length) return;
          const [planBody, groupBody] = await Promise.all([
            request({ method: "GET", url: buildV2("plan/fetch") }),
            request({ method: "GET", url: buildV2("server/group/fetch") })
          ]);
          state.opsCache.plans = normalizeList(planBody);
          state.opsCache.groups = normalizeList(groupBody);
          return;
        }

        if (moduleKey === "payments") {
          if (!force && state.opsCache.payments.length) return;
          const [configBody, paymentBody, methodsBody] = await Promise.all([
            ensureOpsConfig(force),
            request({ method: "GET", url: buildV2("payment/fetch") }),
            request({ method: "GET", url: buildV2("payment/getPaymentMethods") })
          ]);
          state.opsCache.config = normalizeObject(configBody);
          state.opsCache.payments = normalizeList(paymentBody);
          state.opsCache.paymentMethods = normalizeList(methodsBody);
          return;
        }

        if (moduleKey === "notices") {
          if (!force && state.opsCache.notices.length) return;
          const body = await request({ method: "GET", url: buildV2("notice/fetch") });
          state.opsCache.notices = normalizeList(body);
          return;
        }

        if (moduleKey === "tickets") {
          if (!force && state.opsCache.tickets.length) return;
          const body = await request({ method: "GET", url: `${buildV2("ticket/fetch")}?current=1&pageSize=20` });
          state.opsCache.tickets = normalizeList(body);
          return;
        }

        if (moduleKey === "coupons") {
          if (!force && state.opsCache.coupons.length) return;
          const body = await request({ method: "GET", url: `${buildV2("coupon/fetch")}?current=1&pageSize=20` });
          state.opsCache.coupons = normalizeList(body);
          return;
        }

        if (moduleKey === "giftcards") {
          if (!force && state.opsCache.giftTemplates.length) return;
          const [templateBody, typeBody, statBody, codeBody] = await Promise.all([
            request({ method: "GET", url: `${buildV2("gift-card/templates")}?per_page=20&page=1` }),
            request({ method: "GET", url: buildV2("gift-card/types") }),
            request({ method: "GET", url: buildV2("gift-card/statistics") }),
            request({ method: "GET", url: `${buildV2("gift-card/codes")}?per_page=20&page=1` })
          ]);
          state.opsCache.giftTemplates = normalizeList(templateBody);
          state.opsCache.giftTypes = normalizeObject(typeBody);
          state.opsCache.giftStats = normalizeObject(statBody);
          state.opsCache.giftCodes = normalizeList(codeBody);
          return;
        }

        if (moduleKey === "plugins") {
          if (!force && state.opsCache.plugins.length) return;
          const body = await request({ method: "GET", url: buildV2("plugin/getPlugins") });
          state.opsCache.plugins = normalizeList(body);
          return;
        }

        if (moduleKey === "system") {
          if (!force && state.opsCache.systemStatus && state.opsCache.queueStats) return;
          const [statusRs, queueRs, logRs, failedRs] = await Promise.allSettled([
            request({ method: "GET", url: buildV2("system/getSystemStatus") }),
            request({ method: "GET", url: buildV2("system/getQueueStats") }),
            request({ method: "GET", url: `${buildV2("system/getSystemLog")}?current=1&page_size=20` }),
            request({ method: "GET", url: `${buildV2("system/getHorizonFailedJobs")}?current=1&page_size=20` })
          ]);

          state.opsCache.systemStatus = statusRs.status === "fulfilled"
            ? normalizeObject(statusRs.value)
            : { _error: statusRs.reason?.message || "读取失败" };
          state.opsCache.queueStats = queueRs.status === "fulfilled"
            ? normalizeObject(queueRs.value)
            : { _error: queueRs.reason?.message || "读取失败" };
          state.opsCache.systemLogs = logRs.status === "fulfilled"
            ? normalizeObject(logRs.value)
            : { data: [], total: 0, _error: logRs.reason?.message || "读取失败" };
          state.opsCache.failedJobs = failedRs.status === "fulfilled"
            ? normalizeObject(failedRs.value)
            : { data: [], total: 0, _error: failedRs.reason?.message || "读取失败" };
          return;
        }

        if (moduleKey === "traffic") {
          const days = Number(state.opsCache.trafficDays || 30);
          if (!force && state.opsCache.trafficStats && state.opsCache.trafficLogs.length) return;
          const [statBody, logsBody] = await Promise.all([
            request({ method: "GET", url: `${buildV2("traffic-reset/stats")}?days=${days}` }),
            request({ method: "GET", url: `${buildV2("traffic-reset/logs")}?per_page=20&page=1` })
          ]);
          state.opsCache.trafficStats = normalizeObject(statBody);
          const logsObj = normalizeObject(logsBody);
          state.opsCache.trafficLogs = Array.isArray(logsObj.data) ? logsObj.data : [];
          state.opsCache.trafficPagination = logsObj.pagination || null;
          return;
        }
      }

      function renderOpsSecurity() {
        const config = state.opsCache.config || {};
        const safe = config.safe || {};
        const whitelist = Array.isArray(safe.email_whitelist_suffix)
          ? safe.email_whitelist_suffix.join("\n")
          : "";
        const registerMode = String(normalizeConfigValue(safe.register_mode, "all"));

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">安全模式</span>
              <span class="num">${yesNo(safe.safe_mode_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">验证码</span>
              <span class="num">${yesNo(safe.captcha_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">当前 PoW 难度</span>
              <span class="num">${escapeHtml(normalizeConfigValue(safe.pow_effective_difficulty, safe.pow_difficulty || 4))}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">访问安全策略</h4>
            <div class="toolbar">
              ${opsBoolField("safe_mode_enable", "安全模式", safe.safe_mode_enable, 3)}
              ${opsBoolField("safe_email_verify", "注册邮箱验证", safe.email_verify, 3)}
              <div class="field span-3">
                <label for="safe_register_mode">注册方式</label>
                <select id="safe_register_mode">
                  <option value="all" ${registerMode === "all" ? "selected" : ""}>邮箱 + OAuth</option>
                  <option value="email_only" ${registerMode === "email_only" ? "selected" : ""}>仅邮箱注册</option>
                  <option value="oauth_only" ${registerMode === "oauth_only" ? "selected" : ""}>仅 OAuth 注册</option>
                  <option value="closed" ${registerMode === "closed" ? "selected" : ""}>关闭注册</option>
                </select>
              </div>
              ${opsBoolField("safe_email_whitelist_enable", "邮箱后缀白名单", safe.email_whitelist_enable, 3)}
              ${opsBoolField("safe_email_gmail_limit_enable", "Gmail 邮箱规则限制", safe.email_gmail_limit_enable, 3)}
              <div class="field span-6">
                <label for="safe_secure_path">后台登录地址</label>
                <input id="safe_secure_path" value="${escapeHtml(normalizeConfigValue(safe.secure_path, securePath))}" />
              </div>
              <div class="field span-6">
                <label for="safe_email_whitelist_suffix">允许后缀（每行一个）</label>
                <textarea id="safe_email_whitelist_suffix" placeholder="gmail.com&#10;qq.com">${escapeHtml(whitelist)}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn ghost" id="opsGenerateSecurePathBtn">一键生成随机地址</button>
              <button class="btn primary" id="opsSecuritySaveBtn">保存安全设置</button>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">登录与注册防护</h4>
            <div class="toolbar">
              ${opsBoolField("safe_register_limit_by_ip_enable", "按网络地址限制注册", safe.register_limit_by_ip_enable, 3)}
              <div class="field span-3">
                <label for="safe_register_limit_count">注册次数上限</label>
                <input id="safe_register_limit_count" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.register_limit_count, 3))}" />
              </div>
              <div class="field span-3">
                <label for="safe_register_limit_expire">统计时长(分钟)</label>
                <input id="safe_register_limit_expire" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.register_limit_expire, 60))}" />
              </div>
              ${opsBoolField("safe_password_limit_enable", "密码错误限制", safe.password_limit_enable, 3)}
              <div class="field span-3">
                <label for="safe_password_limit_count">密码错误上限</label>
                <input id="safe_password_limit_count" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.password_limit_count, 5))}" />
              </div>
              <div class="field span-3">
                <label for="safe_password_limit_expire">锁定时长(分钟)</label>
                <input id="safe_password_limit_expire" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.password_limit_expire, 60))}" />
              </div>
              <div class="field span-3">
                <label for="safe_login_token_expire_days">登录态有效期(天，0=永不过期)</label>
                <input id="safe_login_token_expire_days" type="number" min="0" max="3650" value="${escapeHtml(normalizeConfigValue(safe.login_token_expire_days, 365))}" />
              </div>
            </div>
          </section>

	          <section class="ops-card">
	            <h4 class="ops-card-title">验证码与防刷验证</h4>
	            <div class="toolbar">
              ${opsBoolField("safe_captcha_enable", "启用验证码", safe.captcha_enable, 3)}
              <div class="field span-3">
                <label for="safe_captcha_type">验证码类型</label>
                <select id="safe_captcha_type">
                  <option value="recaptcha" ${String(safe.captcha_type) === "recaptcha" ? "selected" : ""}>Google 验证（常规）</option>
                  <option value="recaptcha-v3" ${String(safe.captcha_type) === "recaptcha-v3" ? "selected" : ""}>Google 验证（无感）</option>
                  <option value="turnstile" ${String(safe.captcha_type) === "turnstile" ? "selected" : ""}>Cloudflare 验证</option>
                </select>
              </div>
              ${opsBoolField("safe_pow_enable", "启用防刷验证", safe.pow_enable, 3)}
              ${opsBoolField("safe_pow_require_ja3", "设备指纹校验", safe.pow_require_ja3, 3)}
              ${opsBoolField("safe_pow_auto_scale_enable", "自动调节难度", safe.pow_auto_scale_enable, 3)}
              <div class="field span-3">
                <label for="safe_pow_difficulty">防刷难度</label>
                <input id="safe_pow_difficulty" type="number" min="1" max="8" value="${escapeHtml(normalizeConfigValue(safe.pow_difficulty, 4))}" />
              </div>
              <div class="field span-3">
                <label for="safe_pow_ttl">有效时间(秒)</label>
                <input id="safe_pow_ttl" type="number" min="30" max="600" value="${escapeHtml(normalizeConfigValue(safe.pow_ttl, 120))}" />
              </div>
              <div class="field span-3">
                <label for="safe_recap_score">无感验证分数线</label>
                <input id="safe_recap_score" type="number" min="0" max="1" step="0.01" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_v3_score_threshold, 0.5))}" />
              </div>
              <div class="field span-3">
                <label for="safe_pow_auto_max_difficulty">自动调节上限</label>
                <input id="safe_pow_auto_max_difficulty" type="number" min="1" max="8" value="${escapeHtml(normalizeConfigValue(safe.pow_auto_max_difficulty, 7))}" />
              </div>
              <div class="field span-3">
                <label for="safe_pow_effective_difficulty">当前生效难度</label>
                <input id="safe_pow_effective_difficulty" value="${escapeHtml(normalizeConfigValue(safe.pow_effective_difficulty, safe.pow_difficulty || 4))}" disabled />
              </div>
              <div class="field span-6">
                <label for="safe_pow_seed_salt">防刷密钥</label>
                <input id="safe_pow_seed_salt" value="${escapeHtml(normalizeConfigValue(safe.pow_seed_salt, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_pow_base_value">防刷基础值</label>
                <input id="safe_pow_base_value" value="${escapeHtml(normalizeConfigValue(safe.pow_base_value, "portal"))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_key">Google 验证服务端密钥</label>
                <input id="safe_recaptcha_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_site_key">Google 验证网页端密钥</label>
                <input id="safe_recaptcha_site_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_site_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_v3_secret_key">Google 无感服务端密钥</label>
                <input id="safe_recaptcha_v3_secret_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_v3_secret_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_v3_site_key">Google 无感网页端密钥</label>
                <input id="safe_recaptcha_v3_site_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_v3_site_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_turnstile_secret_key">Cloudflare 服务端密钥</label>
                <input id="safe_turnstile_secret_key" value="${escapeHtml(normalizeConfigValue(safe.turnstile_secret_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_turnstile_site_key">Cloudflare 网页端密钥</label>
                <input id="safe_turnstile_site_key" value="${escapeHtml(normalizeConfigValue(safe.turnstile_site_key, ""))}" />
              </div>
	            </div>
	            <p class="hint-text">提示：如果修改后台登录地址，请先确认新地址可打开，再关闭当前页面。</p>
	          </section>
	          <pre id="opsSecurityResult">等待保存...</pre>
	        `;
	      }

      function renderOpsOAuth() {
        const config = state.opsCache.config || {};
        const oauth = config.oauth || {};
        const site = config.site || {};
        const appUrl = String(normalizeConfigValue(site.app_url, settings.base_url || "")).replace(/\/+$/, "");
        const callbackPreview = appUrl
          ? `${appUrl}/api/v1/passport/oauth2/linux-do/callback`
          : "/api/v1/passport/oauth2/linux-do/callback";

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">第三方登录</span>
              <span class="num">${yesNo(oauth.oauth_linux_do_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">客户端编号</span>
              <span class="num">${oauth.oauth_linux_do_client_id ? "已填写" : "未填写"}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">客户端密钥</span>
              <span class="num">${oauth.oauth_linux_do_client_secret ? "已填写" : "未填写"}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">Linux DO 登录设置</h4>
            <div class="toolbar">
              ${opsBoolField("oauth_linux_do_enable", "启用 Linux DO 登录", oauth.oauth_linux_do_enable, 4)}
              <div class="field span-8">
                <label for="oauth_linux_do_client_id">客户端编号（Client ID）</label>
                <input id="oauth_linux_do_client_id" value="${escapeHtml(normalizeConfigValue(oauth.oauth_linux_do_client_id, ""))}" placeholder="请填写 Linux DO 应用 Client ID" />
              </div>
              <div class="field span-12">
                <label for="oauth_linux_do_client_secret">客户端密钥（Client Secret）</label>
                <input id="oauth_linux_do_client_secret" type="password" value="${escapeHtml(normalizeConfigValue(oauth.oauth_linux_do_client_secret, ""))}" placeholder="请填写 Linux DO 应用 Client Secret" />
              </div>
              <div class="field span-12">
                <label for="oauth_linux_do_redirect_uri">回调地址（必须和 Linux DO 应用后台一致）</label>
                <input id="oauth_linux_do_redirect_uri" value="${escapeHtml(normalizeConfigValue(oauth.oauth_linux_do_redirect_uri, callbackPreview))}" />
              </div>
              <div class="field span-12">
                <label>系统推荐回调地址（根据站点地址自动生成）</label>
                <input value="${escapeHtml(callbackPreview)}" disabled />
              </div>
            </div>
            <div class="actions">
              <button class="btn ghost" id="opsOAuthUsePreviewBtn">使用推荐回调地址</button>
              <button class="btn primary" id="opsOAuthSaveBtn">保存第三方登录设置</button>
            </div>
            <p class="hint-text">普通用户会显示第三方登录得到的用户名，建议先配置完整后再开放登录入口。</p>
          </section>

          <pre id="opsOAuthResult">等待保存...</pre>
        `;
      }

      function renderOpsSite() {
        const config = state.opsCache.config || {};
        const site = config.site || {};
        const frontend = config.frontend || {};
        const subscribe = config.subscribe || {};
        const currentTheme = normalizeConfigValue(frontend.frontend_theme, "Maintainable");

        return `
          <section class="ops-card">
            <h4 class="ops-card-title">站点基础信息</h4>
            <div class="toolbar">
              <div class="field span-6">
                <label for="site_app_name">站点名称</label>
                <input id="site_app_name" value="${escapeHtml(normalizeConfigValue(site.app_name, "Portal"))}" />
              </div>
              <div class="field span-6">
                <label for="site_app_description">站点描述</label>
                <input id="site_app_description" value="${escapeHtml(normalizeConfigValue(site.app_description, ""))}" />
              </div>
              <div class="field span-6">
                <label for="site_app_url">站点地址</label>
                <input id="site_app_url" value="${escapeHtml(normalizeConfigValue(site.app_url, ""))}" placeholder="https://example.com" />
              </div>
              <div class="field span-6">
                <label for="site_subscribe_url">订阅域名（可空）</label>
                <input id="site_subscribe_url" value="${escapeHtml(normalizeConfigValue(site.subscribe_url, ""))}" placeholder="https://sub.example.com" />
              </div>
              <div class="field span-6">
                <label for="site_logo">Logo 地址</label>
                <input id="site_logo" value="${escapeHtml(normalizeConfigValue(site.logo, ""))}" />
              </div>
              <div class="field span-6">
                <label for="site_tos_url">服务条款地址</label>
                <input id="site_tos_url" value="${escapeHtml(normalizeConfigValue(site.tos_url, ""))}" />
              </div>
              ${opsBoolField("site_force_https", "强制 HTTPS", site.force_https, 3)}
              <div class="field span-3">
                <label for="site_currency">货币代码</label>
                <input id="site_currency" value="${escapeHtml(normalizeConfigValue(site.currency, "CNY"))}" />
              </div>
              <div class="field span-3">
                <label for="site_currency_symbol">货币符号</label>
                <input id="site_currency_symbol" value="${escapeHtml(normalizeConfigValue(site.currency_symbol, "¥"))}" />
              </div>
              <div class="field span-3">
                <label for="site_try_out_plan_id">试用套餐编号</label>
                <input id="site_try_out_plan_id" type="number" min="0" value="${escapeHtml(normalizeConfigValue(site.try_out_plan_id, 0))}" />
              </div>
              <div class="field span-3">
                <label for="site_try_out_hour">试用时长(小时)</label>
                <input id="site_try_out_hour" type="number" min="1" value="${escapeHtml(normalizeConfigValue(site.try_out_hour, 1))}" />
              </div>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">前端与订阅行为</h4>
            <div class="toolbar">
              <div class="field span-4">
                <label for="site_frontend_theme_fixed">前端主题（固定）</label>
                <input id="site_frontend_theme_fixed" value="${escapeHtml(currentTheme)}" disabled />
              </div>
              <div class="field span-4">
                <label for="site_frontend_sidebar">侧边栏风格</label>
                <select id="site_frontend_sidebar">
                  <option value="light" ${String(frontend.frontend_theme_sidebar) === "light" ? "selected" : ""}>light</option>
                  <option value="dark" ${String(frontend.frontend_theme_sidebar) === "dark" ? "selected" : ""}>dark</option>
                </select>
              </div>
              <div class="field span-4">
                <label for="site_frontend_header">顶部风格</label>
                <select id="site_frontend_header">
                  <option value="dark" ${String(frontend.frontend_theme_header) === "dark" ? "selected" : ""}>dark</option>
                  <option value="light" ${String(frontend.frontend_theme_header) === "light" ? "selected" : ""}>light</option>
                </select>
              </div>
              <div class="field span-4">
                <label for="site_frontend_color">主题配色</label>
                <select id="site_frontend_color">
                  <option value="default" ${String(frontend.frontend_theme_color) === "default" ? "selected" : ""}>default</option>
                  <option value="darkblue" ${String(frontend.frontend_theme_color) === "darkblue" ? "selected" : ""}>darkblue</option>
                  <option value="black" ${String(frontend.frontend_theme_color) === "black" ? "selected" : ""}>black</option>
                  <option value="green" ${String(frontend.frontend_theme_color) === "green" ? "selected" : ""}>green</option>
                </select>
              </div>
              <div class="field span-4">
                <label for="site_subscribe_path">订阅路径</label>
                <input id="site_subscribe_path" value="${escapeHtml(normalizeConfigValue(subscribe.subscribe_path, "s"))}" />
              </div>
              <div class="field span-12">
                <label for="site_subscribe_root_domains">订阅根域名（每行一个，用于随机子域名）</label>
                <textarea id="site_subscribe_root_domains" placeholder="example.com&#10;sub.example.net">${escapeHtml(normalizeConfigValue(site.subscribe_root_domains, ""))}</textarea>
              </div>
              ${opsBoolField("site_plan_change_enable", "允许变更套餐", subscribe.plan_change_enable, 4)}
              ${opsBoolField("site_surplus_enable", "保留剩余价值", subscribe.surplus_enable, 4)}
              <div class="field span-12">
                <label for="site_frontend_background_url">背景图地址（可空）</label>
                <input id="site_frontend_background_url" value="${escapeHtml(normalizeConfigValue(frontend.frontend_background_url, ""))}" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsSiteSaveBtn">保存站点设置</button>
            </div>
          </section>

          <pre id="opsSiteResult">等待保存...</pre>
        `;
      }

      function renderOpsTelegram() {
        const config = state.opsCache.config || {};
        const telegram = config.telegram || {};

        return `
          <section class="ops-card">
            <h4 class="ops-card-title">Bot 基础配置</h4>
            <div class="toolbar">
              ${opsBoolField("telegram_bot_enable", "启用 Telegram Bot", telegram.telegram_bot_enable, 3)}
              <div class="field span-6">
                <label for="telegram_bot_token">Bot Token</label>
                <input id="telegram_bot_token" type="password" autocomplete="new-password" value="${escapeHtml(normalizeConfigValue(telegram.telegram_bot_token, ""))}" placeholder="Bot Token" />
              </div>
              <div class="field span-6">
                <label for="telegram_discuss_link">讨论群/频道链接</label>
                <input id="telegram_discuss_link" value="${escapeHtml(normalizeConfigValue(telegram.telegram_discuss_link, ""))}" placeholder="https://t.me/..." />
              </div>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">功能与通知</h4>
            <div class="toolbar">
              ${opsBoolField("telegram_user_ticket_enable", "允许用户在 Telegram 处理工单", telegram.telegram_user_ticket_enable, 3)}
              ${opsBoolField("telegram_notify_ticket_created", "工单创建通知", telegram.telegram_notify_ticket_created, 3)}
              ${opsBoolField("telegram_notify_ticket_replied", "工单回复通知", telegram.telegram_notify_ticket_replied, 3)}
              ${opsBoolField("telegram_notify_ticket_closed", "工单关闭通知", telegram.telegram_notify_ticket_closed, 3)}
              ${opsBoolField("telegram_notify_payment_success", "支付成功通知", telegram.telegram_notify_payment_success, 3)}
              ${opsBoolField("telegram_notify_notice_published", "公告发布通知", telegram.telegram_notify_notice_published, 3)}
              ${opsBoolField("telegram_notify_tcping_alert", "TCPing 告警通知", telegram.telegram_notify_tcping_alert, 3)}
              ${opsBoolField("telegram_notify_tcping_recover", "TCPing 恢复通知", telegram.telegram_notify_tcping_recover, 3)}
              ${opsBoolField("telegram_notify_refund_vote", "争议投票通知", telegram.telegram_notify_refund_vote, 3)}
              ${opsBoolField("telegram_notify_refund_status", "争议状态变更通知", telegram.telegram_notify_refund_status, 3)}
              ${opsBoolField("telegram_notify_user_risk_detected", "用户风险审查通知", telegram.telegram_notify_user_risk_detected, 3)}
              ${opsBoolField("telegram_notify_user_banned", "用户封禁通知", telegram.telegram_notify_user_banned, 3)}
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">操作</h4>
            <div class="toolbar">
              <div class="field span-12">
                <div class="actions">
                  <button class="btn ghost" id="opsRegisterTelegramWebhookBtn">注册/刷新 Webhook</button>
                  <button class="btn primary" id="opsTelegramSaveBtn">保存 Telegram 设置</button>
                </div>
              </div>
            </div>
            <pre id="opsTelegramResult">等待操作...</pre>
          </section>
        `;
      }

      function renderOpsRiskReview() {
        const config = state.opsCache.config || {};
        const risk = config.risk_review || {};
        const telegram = config.telegram || {};
        const reviews = Array.isArray(state.opsCache.riskReviews) ? state.opsCache.riskReviews : [];
        const pagination = state.opsCache.riskReviewPagination || {};
        const banRecords = Array.isArray(state.opsCache.banRecords) ? state.opsCache.banRecords : [];
        const banPagination = state.opsCache.banRecordPagination || {};
        const reviewTotal = Number(pagination.total || reviews.length || 0);
        const banTotal = Number(banPagination.total || banRecords.length || 0);

        return `
          <section class="ops-card">
            <p class="ops-section-kicker">RISK REVIEW</p>
            <h4 class="ops-card-title">共享 IP 风险识别与封禁流水</h4>
            <p class="ops-card-subtitle">当单 IP 命中多个用户时先进入待审队列，LLM 只做预审建议，最终封禁动作保留给超级管理员人工确认。</p>
            <div class="ops-main-head-meta">
              <span class="ops-module-chip"><strong>Review</strong> ${escapeHtml(yesNo(risk.user_risk_review_enable))}</span>
              <span class="ops-module-chip"><strong>LLM</strong> ${escapeHtml(yesNo(risk.user_risk_review_llm_enable))}</span>
              <span class="ops-module-chip"><strong>Cooldown</strong> ${escapeHtml(String(normalizeConfigValue(risk.user_risk_review_notify_cooldown_minutes, 60)))} 分钟</span>
            </div>
          </section>

          <div class="ops-highlight-grid ops-grid-4">
            <article class="ops-kpi">
              <span class="label">审查开关</span>
              <span class="num">${yesNo(risk.user_risk_review_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">LLM 预审查</span>
              <span class="num">${yesNo(risk.user_risk_review_llm_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">最近审查记录</span>
              <span class="num">${formatCompactNumber(reviewTotal)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">封禁记录总数</span>
              <span class="num">${formatCompactNumber(banTotal)}</span>
            </article>
          </div>

          <div class="ops-split-grid">
            <section class="ops-card">
              <div class="ops-table-header">
                <div class="ops-table-caption">
                  <h4 class="ops-card-title">共享 IP 风险识别</h4>
                  <p class="ops-card-subtitle">控制扫描窗口、命中人数阈值、提醒冷却和 Telegram 通知节奏。</p>
                </div>
              </div>
              <div class="toolbar">
                ${opsBoolField("risk_user_risk_review_enable", "启用定期审查", risk.user_risk_review_enable, 3)}
                <div class="field span-3">
                  <label for="risk_user_risk_review_schedule_minutes">审查间隔(分钟)</label>
                  <input id="risk_user_risk_review_schedule_minutes" type="number" min="5" max="1440" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_schedule_minutes, 30))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_time_window_minutes">共享 IP 统计窗口(分钟)</label>
                  <input id="risk_user_risk_review_time_window_minutes" type="number" min="5" max="1440" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_time_window_minutes, 60))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_context_hours">上下文分析时长(小时)</label>
                  <input id="risk_user_risk_review_context_hours" type="number" min="1" max="168" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_context_hours, 24))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_min_shared_ip_users">最少命中用户数</label>
                  <input id="risk_user_risk_review_min_shared_ip_users" type="number" min="2" max="50" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_min_shared_ip_users, 2))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_scan_limit">单次最大扫描 IP 数</label>
                  <input id="risk_user_risk_review_scan_limit" type="number" min="1" max="200" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_scan_limit, 20))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_notify_cooldown_minutes">提醒冷却(分钟)</label>
                  <input id="risk_user_risk_review_notify_cooldown_minutes" type="number" min="5" max="10080" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_notify_cooldown_minutes, 60))}" />
                </div>
                <div class="field span-6">
                  <label>通知状态</label>
                  <div class="actions">
                    <span class="badge ${isOn(telegram.telegram_notify_user_risk_detected) ? "ok" : "warn"}">风险提醒 ${yesNo(telegram.telegram_notify_user_risk_detected)}</span>
                    <span class="badge ${isOn(telegram.telegram_notify_user_banned) ? "ok" : "warn"}">封禁通知 ${yesNo(telegram.telegram_notify_user_banned)}</span>
                    <button class="btn ghost" id="opsRiskNotificationsBtn" type="button">通知设置</button>
                  </div>
                </div>
              </div>
            </section>

            <section class="ops-card">
              <div class="ops-table-header">
                <div class="ops-table-caption">
                  <h4 class="ops-card-title">LLM 预审查与人工执行</h4>
                  <p class="ops-card-subtitle">模型只给出滥用建议与封禁意见，实际封禁仍需人工确认并填写原因。</p>
                </div>
                <div class="actions">
                  <button class="btn primary" id="opsRiskReviewSaveBtn">保存设置</button>
                  <button class="btn ghost" id="opsRiskReviewRunBtn">立即执行一次审查</button>
                  <button class="btn ghost" id="opsRiskReviewReloadBtn">刷新审查记录</button>
                </div>
              </div>
              <div class="toolbar">
                ${opsBoolField("risk_user_risk_review_llm_enable", "启用 LLM 预审", risk.user_risk_review_llm_enable, 3)}
                <div class="field span-6">
                  <label for="risk_user_risk_review_llm_base_url">OpenAI Compatible Base URL</label>
                  <input id="risk_user_risk_review_llm_base_url" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_base_url, ""))}" placeholder="https://api.openai.com/v1 或 https://api.openai.com/v1/responses" />
                </div>
                <div class="field span-6">
                  <label for="risk_user_risk_review_llm_api_key">API Key</label>
                  <input id="risk_user_risk_review_llm_api_key" type="password" autocomplete="new-password" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_api_key, ""))}" placeholder="sk-..." />
                </div>
                <p class="hint-text span-12">支持直接填写根路径、完整 <code>/chat/completions</code> 地址，或完整 <code>/responses</code> 地址，系统会自动识别端点类型。</p>
                <div class="field span-6">
                  <label for="risk_user_risk_review_llm_model">模型名称</label>
                  <input id="risk_user_risk_review_llm_model" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_model, ""))}" placeholder="gpt-4.1-mini / qwen-plus / deepseek-chat" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_llm_timeout_seconds">超时(秒)</label>
                  <input id="risk_user_risk_review_llm_timeout_seconds" type="number" min="5" max="120" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_timeout_seconds, 20))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_llm_temperature">Temperature</label>
                  <input id="risk_user_risk_review_llm_temperature" type="number" min="0" max="1" step="0.1" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_temperature, 0.2))}" />
                </div>
              </div>
              <div class="ops-helper-list">
                <div class="ops-helper-item">
                  <strong>人工复核优先级</strong>
                  <p>单 IP 命中多用户会直接进入中风险，建议先看共享 IP、匹配用户和模型摘要，再决定是否封禁。</p>
                </div>
                <div class="ops-helper-item">
                  <strong>通知策略</strong>
                  <p>Telegram 仅负责提醒与封禁结果同步，避免相同用户在冷却期内重复打扰。</p>
                </div>
              </div>
            </section>
          </div>

          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">最近审查结果</h4>
                <p class="ops-card-subtitle">当前页 ${pagination.current_page || 1} / ${pagination.last_page || 1}，支持直接从建议结果进入封禁动作。</p>
              </div>
              <span class="badge ok">共 ${formatCompactNumber(reviewTotal)} 条审查记录</span>
            </div>
            <div class="table-wrap">
              <table class="ops-review-table">
                <thead>
                  <tr>
                    <th>时间</th>
                    <th>用户</th>
                    <th>共享 IP</th>
                    <th>命中用户</th>
                    <th>风险</th>
                    <th>状态</th>
                    <th>模型</th>
                    <th>意见摘要</th>
                    <th>处理建议</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody>
                  ${reviews.length ? reviews.map((review) => `
                    <tr>
                      <td>${formatAnyTimestamp(review.reviewed_at)}</td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>#${review.user_id}</strong>
                          <span>${escapeHtml(review.user_email || "-")}</span>
                        </div>
                      </td>
                      <td>${escapeHtml(review.shared_ip || "-")}</td>
                      <td>${escapeHtml((review.matched_users || []).map((item) => item.email || `#${item.id}`).join(", ") || String(review.matched_user_count || 0))}</td>
                      <td><span class="badge ${String(review.risk_level) === "high" ? "warn" : "ok"}">${escapeHtml(review.risk_level || "medium")} / ${escapeHtml(review.suspicion_score || 0)}</span></td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong><span class="badge ${review.user_banned ? "warn" : "ok"}">${review.user_banned ? "已封禁" : "正常"}</span></strong>
                          <span>${escapeHtml((review.user_ban_reason || "").trim() || "-")}</span>
                        </div>
                      </td>
                      <td>${escapeHtml(review.llm_model || "heuristic")}</td>
                      <td>${escapeHtml(review.summary || "-")}</td>
                      <td>${escapeHtml(review.recommendation || "-")}</td>
                      <td>
                        ${review.user_banned ? '<span class="badge ok">已处理</span>' : `<button class="btn warn risk-review-ban-btn" data-user-id="${review.user_id}" data-user-email="${escapeHtml(review.user_email || "")}" data-ban-reason="${escapeHtml(((review.recommendation || review.summary || "").trim()))}">封禁用户</button>`}
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="10" class="empty">暂无风险审查记录</td></tr>'}
                </tbody>
              </table>
            </div>
            <div class="actions ops-paged-actions">
              <button class="btn ghost" id="opsRiskReviewPrevBtn" ${Number(pagination.current_page || 1) <= 1 ? "disabled" : ""}>上一页</button>
              <button class="btn ghost" id="opsRiskReviewNextBtn" ${Number(pagination.current_page || 1) >= Number(pagination.last_page || 1) ? "disabled" : ""}>下一页</button>
            </div>
          </section>

          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">最近封禁记录</h4>
                <p class="ops-card-subtitle">当前页 ${banPagination.current_page || 1} / ${banPagination.last_page || 1}，记录封禁和解封原因，便于后续追溯。</p>
              </div>
              <span class="badge ok">共 ${formatCompactNumber(banTotal)} 条封禁流水</span>
            </div>
            <div class="table-wrap">
              <table class="ops-review-table">
                <thead>
                  <tr>
                    <th>时间</th>
                    <th>用户</th>
                    <th>操作人</th>
                    <th>动作</th>
                    <th>来源</th>
                    <th>原因</th>
                  </tr>
                </thead>
                <tbody>
                  ${banRecords.length ? banRecords.map((record) => `
                    <tr>
                      <td>${formatAnyTimestamp(record.created_at)}</td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>#${record.user_id}</strong>
                          <span>${escapeHtml(record.user_email || "-")}</span>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${record.admin_id ? `#${record.admin_id}` : "-"}</strong>
                          <span>${escapeHtml(record.admin_email || "system")}</span>
                        </div>
                      </td>
                      <td><span class="badge ${String(record.action) === "ban" ? "warn" : "ok"}">${escapeHtml(record.action || "-")}</span></td>
                      <td>${escapeHtml(record.source || "-")}</td>
                      <td>${escapeHtml(record.reason || "-")}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无封禁记录</td></tr>'}
                </tbody>
              </table>
            </div>
            <div class="actions ops-paged-actions">
              <button class="btn ghost" id="opsBanRecordPrevBtn" ${Number(banPagination.current_page || 1) <= 1 ? "disabled" : ""}>上一页</button>
              <button class="btn ghost" id="opsBanRecordNextBtn" ${Number(banPagination.current_page || 1) >= Number(banPagination.last_page || 1) ? "disabled" : ""}>下一页</button>
            </div>
          </section>

          <pre id="opsRiskReviewResult">等待操作...</pre>
        `;
      }

      function renderOpsPlans() {
        const plans = state.opsCache.plans || [];
        const groups = state.opsCache.groups || [];
        const visiblePlans = plans.filter((item) => isOn(item.show)).length;
        const sellPlans = plans.filter((item) => isOn(item.sell)).length;
        const groupCount = new Set(plans.map((item) => item.group_id).filter(Boolean)).size;
        const groupOptions = [`<option value="">未指定</option>`]
          .concat(groups.map((g) => `<option value="${g.id}">${escapeHtml(g.name || `Group-${g.id}`)}</option>`))
          .join("");

        return `
          <section class="ops-card">
            <p class="ops-section-kicker">PLAN MANAGEMENT</p>
            <h4 class="ops-card-title">套餐管理工作台</h4>
            <p class="ops-card-subtitle">把套餐列表、上下架和编辑器放到一个连续流程里，减少超管维护价格和销售状态时的来回切换。</p>
            <div class="ops-main-head-meta">
              <span class="ops-module-chip"><strong>Total</strong> ${formatCompactNumber(plans.length)} 套餐</span>
              <span class="ops-module-chip"><strong>Visible</strong> ${formatCompactNumber(visiblePlans)}</span>
              <span class="ops-module-chip"><strong>Selling</strong> ${formatCompactNumber(sellPlans)}</span>
            </div>
          </section>

          <div class="ops-summary-grid">
            <article class="ops-summary-card">
              <span>套餐总数</span>
              <strong>${formatCompactNumber(plans.length)}</strong>
              <small>当前已创建的套餐条目总量。</small>
            </article>
            <article class="ops-summary-card">
              <span>展示中</span>
              <strong>${formatCompactNumber(visiblePlans)}</strong>
              <small>前端仍然对用户显示的套餐数。</small>
            </article>
            <article class="ops-summary-card">
              <span>在售套餐</span>
              <strong>${formatCompactNumber(sellPlans)}</strong>
              <small>允许新购或续费的套餐数。</small>
            </article>
            <article class="ops-summary-card">
              <span>关联分组</span>
              <strong>${formatCompactNumber(groupCount)}</strong>
              <small>已在套餐中实际使用的节点分组数量。</small>
            </article>
          </div>

          <div class="ops-split-grid">
          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">套餐列表</h4>
                <p class="ops-card-subtitle">左侧快速浏览套餐当前展示与销售状态，点击编辑可直接回填右侧编辑器。</p>
              </div>
              <div class="actions">
                <button class="btn ghost" id="opsPlansReloadBtn">刷新套餐</button>
                <span class="badge ok">共 ${plans.length} 个套餐</span>
              </div>
            </div>
            <div class="table-wrap">
              <table class="ops-plan-table">
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>分组</th>
                    <th>流量配额</th>
                    <th>限速</th>
                    <th>状态</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsPlansTableBody">
                  ${plans.length ? plans.map((p) => `
                    <tr>
                      <td>${p.id}</td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml(p.name || "-")}</strong>
                          <span>${Array.isArray(p.tags) && p.tags.length ? escapeHtml(p.tags.join(", ")) : "无标签"}</span>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml((p.group && p.group.name) || "-")}</strong>
                          <small>group_id: ${escapeHtml(String(p.group_id || "-"))}</small>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml(String(p.transfer_enable ?? "-"))}</strong>
                          <small>原始配额值</small>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml(String(p.speed_limit || 0))} Mbps</strong>
                          <small>设备 ${escapeHtml(String(p.device_limit || 0))} / 容量 ${escapeHtml(String(p.capacity_limit || 0))}</small>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong><span class="badge ${isOn(p.show) ? "ok" : "warn"}">${isOn(p.show) ? "展示" : "隐藏"}</span></strong>
                          <small>${isOn(p.sell) ? "在售中" : "停售中"}</small>
                          <div class="ops-plan-meter">
                            <div class="ops-plan-meter-fill" style="width:${(isOn(p.show) ? 50 : 0) + (isOn(p.sell) ? 50 : 0)}%"></div>
                          </div>
                        </div>
                      </td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-plan-action="edit" data-id="${p.id}">编辑</button>
                          <button class="btn ghost" data-plan-action="toggle-show" data-id="${p.id}">${isOn(p.show) ? "隐藏" : "展示"}</button>
                          <button class="btn ghost" data-plan-action="toggle-sell" data-id="${p.id}">${isOn(p.sell) ? "停售" : "上架"}</button>
                          <button class="btn danger" data-plan-action="drop" data-id="${p.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无套餐</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">套餐编辑器</h4>
                <p class="ops-card-subtitle">支持新建、回填编辑、价格 JSON 调整和基础销售参数维护。</p>
              </div>
              <div class="actions">
                <button class="btn primary" id="opsPlanSaveBtn">保存套餐</button>
                <button class="btn ghost" id="opsPlanClearBtn">清空编辑器</button>
              </div>
            </div>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsPlanId">套餐编号（新建留空）</label>
                <input id="opsPlanId" type="number" min="1" placeholder="自动创建" />
              </div>
              <div class="field span-6">
                <label for="opsPlanName">套餐名称</label>
                <input id="opsPlanName" placeholder="例如：旗舰套餐" />
              </div>
              <div class="field span-3">
                <label for="opsPlanGroupId">分组</label>
                <select id="opsPlanGroupId">${groupOptions}</select>
              </div>
              <div class="field span-3">
                <label for="opsPlanTransfer">流量配额(GB)</label>
                <input id="opsPlanTransfer" type="number" min="1" value="100" />
              </div>
              <div class="field span-3">
                <label for="opsPlanSpeedLimit">速度限制(Mbps)</label>
                <input id="opsPlanSpeedLimit" type="number" min="0" value="0" />
              </div>
              <div class="field span-3">
                <label for="opsPlanDeviceLimit">设备限制</label>
                <input id="opsPlanDeviceLimit" type="number" min="0" value="0" />
              </div>
              <div class="field span-3">
                <label for="opsPlanCapacityLimit">容量限制(人数)</label>
                <input id="opsPlanCapacityLimit" type="number" min="0" value="0" />
              </div>
              <div class="field span-4">
                <label for="opsPlanResetMethod">重置策略</label>
                <input id="opsPlanResetMethod" type="number" min="0" value="0" />
              </div>
              <div class="field span-8">
                <label for="opsPlanTags">标签（逗号分隔）</label>
                <input id="opsPlanTags" placeholder="热门,推荐" />
              </div>
              <div class="field span-12">
                <label for="opsPlanContent">套餐描述</label>
                <textarea id="opsPlanContent" placeholder="可写套餐文案说明"></textarea>
              </div>
              <div class="field span-12">
                <label for="opsPlanPrices">价格设置（按示例填写，单位分）</label>
                <textarea id="opsPlanPrices">{}</textarea>
              </div>
            </div>
            <div class="ops-helper-list">
              <div class="ops-helper-item">
                <strong>编辑流程</strong>
                <p>先从左侧点击“编辑”回填已有套餐，再统一保存，避免手动抄写字段造成误差。</p>
              </div>
              <div class="ops-helper-item">
                <strong>价格字段</strong>
                <p>prices 保持 JSON 结构即可，适合一次性批量调整月付、年付和重置流量价格。</p>
              </div>
            </div>
            <pre id="opsPlanResult">等待操作...</pre>
          </section>
          </div>
        `;
      }

      function renderOpsPayments() {
        const payments = state.opsCache.payments || [];
        const methods = state.opsCache.paymentMethods || [];
        const system = (state.opsCache.config || {}).system || {};
        const methodOptions = methods.length
          ? methods.map((m) => `<option value="${escapeHtml(m)}">${escapeHtml(m)}</option>`).join("")
          : '<option value="">暂无可用支付方式</option>';

        return `
          <section class="ops-card">
            <div class="ops-table-header">
              <div>
                <h4 class="ops-card-title">退款与争议</h4>
                <p class="ops-card-subtitle">用户争议投票入口与接口策略</p>
              </div>
              <button class="btn primary" id="opsPaymentPolicySaveBtn" type="button">保存策略</button>
            </div>
            <div class="toolbar">
              ${opsBoolField("opsRefundDisputeEnable", "启用争议退款投票", system.refund_dispute_enable, 4)}
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">支付方式列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsPaymentsReloadBtn">刷新支付方式</button>
              <span class="badge ok">共 ${payments.length} 项</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>支付标识</th>
                    <th>状态</th>
                    <th>手续费</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsPaymentsTableBody">
                  ${payments.length ? payments.map((p) => `
                    <tr>
                      <td>${p.id}</td>
                      <td>${escapeHtml(p.name || "-")}</td>
                      <td>${escapeHtml(p.payment || "-")}</td>
                      <td><span class="badge ${isOn(p.enable) ? "ok" : "warn"}">${isOn(p.enable) ? "启用" : "停用"}</span></td>
                      <td>${escapeHtml(p.handling_fee_fixed || 0)} / ${escapeHtml(p.handling_fee_percent || 0)}%</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-payment-action="edit" data-id="${p.id}">编辑</button>
                          <button class="btn ghost" data-payment-action="toggle" data-id="${p.id}">${isOn(p.enable) ? "停用" : "启用"}</button>
                          <button class="btn danger" data-payment-action="drop" data-id="${p.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无支付方式</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">支付方式编辑器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsPaymentId">编号（新建留空）</label>
                <input id="opsPaymentId" type="number" min="1" placeholder="自动创建" />
              </div>
              <div class="field span-5">
                <label for="opsPaymentName">显示名称</label>
                <input id="opsPaymentName" placeholder="例如：支付宝" />
              </div>
              <div class="field span-4">
                <label for="opsPaymentGateway">支付方式标识</label>
                <select id="opsPaymentGateway">${methodOptions}</select>
              </div>
              <div class="field span-4">
                <label for="opsPaymentIcon">图标</label>
                <input id="opsPaymentIcon" placeholder="alipay" />
              </div>
              <div class="field span-4">
                <label for="opsPaymentNotifyDomain">回调域名</label>
                <input id="opsPaymentNotifyDomain" placeholder="https://example.com" />
              </div>
              <div class="field span-2">
                <label for="opsPaymentFeeFixed">固定手续费</label>
                <input id="opsPaymentFeeFixed" type="number" min="0" value="0" />
              </div>
              <div class="field span-2">
                <label for="opsPaymentFeePercent">百分比手续费</label>
                <input id="opsPaymentFeePercent" type="number" min="0" max="100" step="0.01" value="0" />
              </div>
              <div class="field span-12">
                <label for="opsPaymentConfig">详细设置（按示例填写）</label>
                <textarea id="opsPaymentConfig">{}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsPaymentSaveBtn">保存支付方式</button>
              <button class="btn ghost" id="opsPaymentClearBtn">清空编辑器</button>
            </div>
          </section>

          <pre id="opsPaymentResult">等待操作...</pre>
        `;
      }

      function renderOpsNotices() {
        const notices = state.opsCache.notices || [];
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">公告列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsNoticesReloadBtn">刷新公告</button>
              <span class="badge ok">共 ${notices.length} 条</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>标题</th>
                    <th>显示</th>
                    <th>弹窗</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsNoticesTableBody">
                  ${notices.length ? notices.map((n) => `
                    <tr>
                      <td>${n.id}</td>
                      <td>${escapeHtml(n.title || "-")}</td>
                      <td>${yesNo(n.show)}</td>
                      <td>${yesNo(n.popup)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-notice-action="edit" data-id="${n.id}">编辑</button>
                          <button class="btn ghost" data-notice-action="toggle" data-id="${n.id}">${isOn(n.show) ? "隐藏" : "展示"}</button>
                          <button class="btn danger" data-notice-action="drop" data-id="${n.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="5" class="empty">暂无公告</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">公告编辑器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsNoticeId">编号（新建留空）</label>
                <input id="opsNoticeId" type="number" min="1" />
              </div>
              <div class="field span-9">
                <label for="opsNoticeTitle">标题</label>
                <input id="opsNoticeTitle" />
              </div>
              <div class="field span-6">
                <label for="opsNoticeImgUrl">图片地址</label>
                <input id="opsNoticeImgUrl" />
              </div>
              <div class="field span-6">
                <label for="opsNoticeTags">标签（逗号分隔）</label>
                <input id="opsNoticeTags" placeholder="系统,公告" />
              </div>
              ${opsBoolField("opsNoticeShow", "显示公告", true, 3)}
              ${opsBoolField("opsNoticePopup", "弹窗展示", false, 3)}
              <div class="field span-12">
                <label for="opsNoticeContent">内容</label>
                <textarea id="opsNoticeContent"></textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsNoticeSaveBtn">保存公告</button>
              <button class="btn ghost" id="opsNoticeClearBtn">清空编辑器</button>
            </div>
          </section>

          <pre id="opsNoticeResult">等待操作...</pre>
        `;
      }

      function renderOpsTickets() {
        const tickets = state.opsCache.tickets || [];
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">工单列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsTicketsReloadBtn">刷新工单</button>
              <span class="badge ok">共 ${tickets.length} 条</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>用户</th>
                    <th>状态</th>
                    <th>回复状态</th>
                    <th>更新时间</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsTicketsTableBody">
                  ${tickets.length ? tickets.map((t) => `
                    <tr>
                      <td>${t.id}</td>
                      <td>${escapeHtml((t.user && t.user.email) || "-")}</td>
                      <td>${escapeHtml(t.status)}</td>
                      <td>${escapeHtml(t.reply_status)}</td>
                      <td>${escapeHtml(t.updated_at || "-")}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-ticket-action="detail" data-id="${t.id}">详情</button>
                          <button class="btn ghost" data-ticket-action="reply" data-id="${t.id}">填写回复</button>
                          <button class="btn warn" data-ticket-action="close" data-id="${t.id}">关闭</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无工单</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">工单处理器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsTicketId">工单编号</label>
                <input id="opsTicketId" type="number" min="1" />
              </div>
              <div class="field span-9">
                <label for="opsTicketReply">回复内容</label>
                <input id="opsTicketReply" placeholder="输入回复内容" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsTicketReplyBtn">发送回复</button>
              <button class="btn warn" id="opsTicketCloseBtn">关闭工单</button>
              <button class="btn ghost" id="opsTicketDetailBtn">读取详情</button>
            </div>
          </section>

          <pre id="opsTicketResult">等待操作...</pre>
          <pre id="opsTicketDetail">工单详情输出...</pre>
        `;
      }

      function renderOpsCoupons() {
        const coupons = state.opsCache.coupons || [];
        const now = Math.floor(Date.now() / 1000);
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">优惠券列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsCouponsReloadBtn">刷新优惠券</button>
              <span class="badge ok">共 ${coupons.length} 条</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>券码</th>
                    <th>类型</th>
                    <th>值</th>
                    <th>显示</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsCouponsTableBody">
                  ${coupons.length ? coupons.map((c) => `
                    <tr>
                      <td>${c.id}</td>
                      <td>${escapeHtml(c.name || "-")}</td>
                      <td>${escapeHtml(c.code || "-")}</td>
                      <td>${Number(c.type) === 2 ? "比例" : "金额"}</td>
                      <td>${escapeHtml(c.value || 0)}</td>
                      <td>${yesNo(c.show)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-coupon-action="toggle" data-id="${c.id}">${isOn(c.show) ? "隐藏" : "展示"}</button>
                          <button class="btn danger" data-coupon-action="drop" data-id="${c.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无优惠券</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">优惠券生成器</h4>
            <div class="toolbar">
              <div class="field span-4">
                <label for="opsCouponName">名称</label>
                <input id="opsCouponName" value="活动券" />
              </div>
              <div class="field span-2">
                <label for="opsCouponType">类型</label>
                <select id="opsCouponType">
                  <option value="1">金额</option>
                  <option value="2">比例</option>
                </select>
              </div>
              <div class="field span-2">
                <label for="opsCouponValue">值</label>
                <input id="opsCouponValue" type="number" value="100" />
              </div>
              <div class="field span-2">
                <label for="opsCouponGenerateCount">生成数量</label>
                <input id="opsCouponGenerateCount" type="number" min="1" max="500" value="1" />
              </div>
              <div class="field span-2">
                <label for="opsCouponCode">固定券码(可空)</label>
                <input id="opsCouponCode" />
              </div>
              <div class="field span-4">
                <label for="opsCouponStartedAt">开始时间</label>
                <input id="opsCouponStartedAt" type="datetime-local" value="${toInputDatetime(now)}" />
              </div>
              <div class="field span-4">
                <label for="opsCouponEndedAt">结束时间</label>
                <input id="opsCouponEndedAt" type="datetime-local" value="${toInputDatetime(now + 86400 * 7)}" />
              </div>
              <div class="field span-2">
                <label for="opsCouponLimitUse">总可用次数</label>
                <input id="opsCouponLimitUse" type="number" min="1" />
              </div>
              <div class="field span-2">
                <label for="opsCouponLimitUseWithUser">单用户次数</label>
                <input id="opsCouponLimitUseWithUser" type="number" min="1" />
              </div>
              <div class="field span-6">
                <label for="opsCouponLimitPlanIds">限制套餐编号（逗号分隔）</label>
                <input id="opsCouponLimitPlanIds" placeholder="1,2,3" />
              </div>
              <div class="field span-6">
                <label for="opsCouponLimitPeriod">限制周期（逗号分隔）</label>
                <input id="opsCouponLimitPeriod" placeholder="month_price,year_price" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsCouponGenerateBtn">生成优惠券</button>
            </div>
          </section>

          <pre id="opsCouponResult">等待操作...</pre>
        `;
      }

      function renderOpsGiftCards() {
        const templates = state.opsCache.giftTemplates || [];
        const codes = state.opsCache.giftCodes || [];
        const types = state.opsCache.giftTypes || {};
        const typeData = normalizeObject(types.data || types);
        const stats = normalizeObject((state.opsCache.giftStats && state.opsCache.giftStats.data) || state.opsCache.giftStats || {});
        const totalStats = stats.total_stats || {};

        const typeOptions = Object.keys(typeData).length
          ? Object.keys(typeData).map((k) => `<option value="${escapeHtml(k)}">${escapeHtml(typeData[k])}</option>`).join("")
          : '<option value="1">类型 1</option>';

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">模板数量</span>
              <span class="num">${money(totalStats.templates_count || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">兑换码总数</span>
              <span class="num">${money(totalStats.codes_count || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">已使用兑换码</span>
              <span class="num">${money(totalStats.used_codes_count || 0)}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">模板管理</h4>
            <div class="actions">
              <button class="btn ghost" id="opsGiftReloadBtn">刷新礼品卡数据</button>
              <span class="badge ok">模板 ${templates.length}</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>类型</th>
                    <th>状态</th>
                    <th>兑换码</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsGiftTemplateTableBody">
                  ${templates.length ? templates.map((t) => `
                    <tr>
                      <td>${t.id}</td>
                      <td>${escapeHtml(t.name || "-")}</td>
                      <td>${escapeHtml(t.type_name || t.type || "-")}</td>
                      <td>${isOn(t.status) ? "启用" : "停用"}</td>
                      <td>${escapeHtml(t.codes_count || 0)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-gift-template-action="edit" data-id="${t.id}">编辑</button>
                          <button class="btn danger" data-gift-template-action="drop" data-id="${t.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无模板</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">模板编辑器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsGiftTemplateId">模板编号（新建留空）</label>
                <input id="opsGiftTemplateId" type="number" min="1" />
              </div>
              <div class="field span-5">
                <label for="opsGiftTemplateName">模板名称</label>
                <input id="opsGiftTemplateName" />
              </div>
              <div class="field span-2">
                <label for="opsGiftTemplateType">类型</label>
                <select id="opsGiftTemplateType">${typeOptions}</select>
              </div>
              ${opsBoolField("opsGiftTemplateStatus", "启用状态", true, 2)}
              <div class="field span-12">
                <label for="opsGiftTemplateDesc">描述</label>
                <textarea id="opsGiftTemplateDesc"></textarea>
              </div>
              <div class="field span-4">
                <label for="opsGiftTemplateThemeColor">主题色</label>
                <input id="opsGiftTemplateThemeColor" value="#1890ff" />
              </div>
              <div class="field span-8">
                <label for="opsGiftTemplateRewards">奖励设置（按示例填写，必填）</label>
                <textarea id="opsGiftTemplateRewards">{"balance":1000}</textarea>
              </div>
              <div class="field span-6">
                <label for="opsGiftTemplateConditions">使用条件（按示例填写）</label>
                <textarea id="opsGiftTemplateConditions">{}</textarea>
              </div>
              <div class="field span-6">
                <label for="opsGiftTemplateLimits">使用限制（按示例填写）</label>
                <textarea id="opsGiftTemplateLimits">{}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsGiftTemplateSaveBtn">保存模板</button>
              <button class="btn ghost" id="opsGiftTemplateClearBtn">清空编辑器</button>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">兑换码管理</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsGiftCodeTemplateId">模板编号</label>
                <input id="opsGiftCodeTemplateId" type="number" min="1" />
              </div>
              <div class="field span-3">
                <label for="opsGiftCodeCount">生成数量</label>
                <input id="opsGiftCodeCount" type="number" min="1" max="10000" value="10" />
              </div>
              <div class="field span-2">
                <label for="opsGiftCodePrefix">前缀</label>
                <input id="opsGiftCodePrefix" value="GC" />
              </div>
              <div class="field span-2">
                <label for="opsGiftCodeHours">有效小时</label>
                <input id="opsGiftCodeHours" type="number" min="1" />
              </div>
              <div class="field span-2">
                <label for="opsGiftCodeMaxUsage">最大次数</label>
                <input id="opsGiftCodeMaxUsage" type="number" min="1" value="1" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsGiftCodeGenerateBtn">生成兑换码</button>
            </div>

            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>兑换码</th>
                    <th>模板</th>
                    <th>状态</th>
                    <th>使用/上限</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsGiftCodeTableBody">
                  ${codes.length ? codes.map((c) => `
                    <tr>
                      <td>${c.id}</td>
                      <td>${escapeHtml(c.code)}</td>
                      <td>${escapeHtml(c.template_name || c.template_id)}</td>
                      <td>${escapeHtml(c.status_name || c.status)}</td>
                      <td>${escapeHtml(c.usage_count || 0)} / ${escapeHtml(c.max_usage || 1)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-gift-code-action="toggle" data-id="${c.id}" data-status="${escapeHtml(c.status)}">${Number(c.status) === 3 ? "启用" : "禁用"}</button>
                          <button class="btn danger" data-gift-code-action="drop" data-id="${c.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无兑换码</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <pre id="opsGiftResult">等待操作...</pre>
        `;
      }

      function renderOpsPlugins() {
        const plugins = state.opsCache.plugins || [];
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">扩展列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsPluginsReloadBtn">刷新扩展</button>
              <span class="badge ok">共 ${plugins.length} 个扩展</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>标识</th>
                    <th>名称</th>
                    <th>版本</th>
                    <th>类型</th>
                    <th>安装</th>
                    <th>启用</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsPluginsTableBody">
                  ${plugins.length ? plugins.map((p) => `
                    <tr>
                      <td>${escapeHtml(p.code)}</td>
                      <td>${escapeHtml(p.name || "-")}</td>
                      <td>${escapeHtml(p.version || "-")}</td>
                      <td>${escapeHtml(p.type || "-")}</td>
                      <td>${yesNo(p.is_installed)}</td>
                      <td>${yesNo(p.is_enabled)}</td>
                      <td>
                        <div class="actions">
                          ${!p.is_installed ? `<button class="btn ghost" data-plugin-action="install" data-code="${escapeHtml(p.code)}">安装</button>` : ""}
                          ${p.is_installed ? `<button class="btn ghost" data-plugin-action="${p.is_enabled ? "disable" : "enable"}" data-code="${escapeHtml(p.code)}">${p.is_enabled ? "禁用" : "启用"}</button>` : ""}
                          ${p.is_installed ? `<button class="btn ghost" data-plugin-action="uninstall" data-code="${escapeHtml(p.code)}">卸载</button>` : ""}
                          ${p.need_upgrade ? `<button class="btn ghost" data-plugin-action="upgrade" data-code="${escapeHtml(p.code)}">升级</button>` : ""}
                          ${p.can_be_deleted ? `<button class="btn danger" data-plugin-action="delete" data-code="${escapeHtml(p.code)}">删除</button>` : ""}
                          <button class="btn ghost" data-plugin-action="config" data-code="${escapeHtml(p.code)}">配置</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无扩展</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">扩展设置编辑器</h4>
            <div class="toolbar">
              <div class="field span-4">
                <label for="opsPluginCode">扩展标识</label>
                <input id="opsPluginCode" placeholder="plugin-code" />
              </div>
              <div class="field span-8">
                <label for="opsPluginConfig">扩展设置（按示例填写）</label>
                <textarea id="opsPluginConfig">{}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsPluginSaveConfigBtn">保存扩展设置</button>
            </div>
          </section>

          <pre id="opsPluginResult">等待操作...</pre>
        `;
      }

      function renderOpsSystem() {
        const status = state.opsCache.systemStatus || {};
        const queue = state.opsCache.queueStats || {};
        const logsObj = state.opsCache.systemLogs || {};
        const failedObj = state.opsCache.failedJobs || {};
        const logs = Array.isArray(logsObj.data) ? logsObj.data : [];
        const failed = Array.isArray(failedObj.data) ? failedObj.data : [];

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">调度状态</span>
              <span class="num">${status._error ? "ERR" : (isOn(status.schedule) ? "ON" : "OFF")}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">队列状态</span>
              <span class="num">${queue._error ? "ERR" : (isOn(queue.status) ? "ON" : "OFF")}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">失败任务</span>
              <span class="num">${money(queue.failedJobs || failedObj.total || 0)}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">系统状态</h4>
            <div class="actions">
              <button class="btn ghost" id="opsSystemReloadBtn">刷新系统数据</button>
            </div>
            <pre id="opsSystemStatusOutput">${escapeHtml(JSON.stringify({ status, queue }, null, 2))}</pre>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">系统日志</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsSystemLogLevel">级别</label>
                <select id="opsSystemLogLevel">
                  <option value="">全部</option>
                  <option value="info">info</option>
                  <option value="warning">warning</option>
                  <option value="error">error</option>
                </select>
              </div>
              <div class="field span-6">
                <label for="opsSystemLogKeyword">关键字</label>
                <input id="opsSystemLogKeyword" placeholder="URI / title / data" />
              </div>
              <div class="field span-3">
                <label>操作</label>
                <div class="actions">
                  <button class="btn primary" id="opsSystemLoadLogsBtn">查询日志</button>
                </div>
              </div>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>级别</th>
                    <th>标题</th>
                    <th>时间</th>
                  </tr>
                </thead>
                <tbody>
                  ${logs.length ? logs.slice(0, 50).map((l) => `
                    <tr>
                      <td>${l.id || "-"}</td>
                      <td>${escapeHtml(l.level || "-")}</td>
                      <td>${escapeHtml(l.title || l.uri || "-")}</td>
                      <td>${formatDateTime(l.created_at)}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="4" class="empty">暂无日志</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">日志清理</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsSystemClearDays">清理多少天前</label>
                <input id="opsSystemClearDays" type="number" min="0" max="365" value="30" />
              </div>
              <div class="field span-3">
                <label for="opsSystemClearLevel">级别</label>
                <select id="opsSystemClearLevel">
                  <option value="all">all</option>
                  <option value="info">info</option>
                  <option value="warning">warning</option>
                  <option value="error">error</option>
                </select>
              </div>
              <div class="field span-3">
                <label for="opsSystemClearLimit">单次清理数量</label>
                <input id="opsSystemClearLimit" type="number" min="100" max="10000" value="1000" />
              </div>
              <div class="field span-3">
                <label>操作</label>
                <div class="actions">
                  <button class="btn warn" id="opsSystemClearBtn">执行清理</button>
                </div>
              </div>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">失败任务</h4>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>连接</th>
                    <th>队列</th>
                    <th>失败时间</th>
                  </tr>
                </thead>
                <tbody>
                  ${failed.length ? failed.slice(0, 30).map((j) => `
                    <tr>
                      <td>${escapeHtml(j.id || "-")}</td>
                      <td>${escapeHtml(j.connection || "-")}</td>
                      <td>${escapeHtml(j.queue || "-")}</td>
                      <td>${escapeHtml(j.failed_at || "-")}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="4" class="empty">暂无失败任务</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <pre id="opsSystemResult">等待操作...</pre>
        `;
      }

      function renderOpsTraffic() {
        const statObj = normalizeObject(state.opsCache.trafficStats || {});
        const stats = normalizeObject(statObj.data || statObj);
        const logs = state.opsCache.trafficLogs || [];
        const pagination = state.opsCache.trafficPagination || {};
        const days = Number(state.opsCache.trafficDays || 30);

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">${days}天总重置</span>
              <span class="num">${money(stats.total_resets || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">手动重置</span>
              <span class="num">${money(stats.manual_resets || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">自动重置</span>
              <span class="num">${money(stats.auto_resets || 0)}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">流量重置控制</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsTrafficDays">统计天数</label>
                <input id="opsTrafficDays" type="number" min="1" max="365" value="${days}" />
              </div>
              <div class="field span-3">
                <label for="opsTrafficUserId">用户编号</label>
                <input id="opsTrafficUserId" type="number" min="1" />
              </div>
              <div class="field span-6">
                <label for="opsTrafficReason">手动重置原因</label>
                <input id="opsTrafficReason" placeholder="例如：申诉处理" />
              </div>
              <div class="field span-3">
                <label>重置操作</label>
                <div class="actions">
                  <button class="btn warn" id="opsTrafficResetBtn">立即重置用户</button>
                </div>
              </div>
              <div class="field span-3">
                <label>用户历史</label>
                <div class="actions">
                  <button class="btn ghost" id="opsTrafficHistoryBtn">读取重置历史</button>
                </div>
              </div>
              <div class="field span-3">
                <label>刷新统计</label>
                <div class="actions">
                  <button class="btn ghost" id="opsTrafficReloadBtn">刷新流量数据</button>
                </div>
              </div>
            </div>
            <pre id="opsTrafficHistory">用户历史输出...</pre>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">重置日志（当前页 ${pagination.current_page || 1} / ${pagination.last_page || 1}）</h4>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>用户</th>
                    <th>类型</th>
                    <th>来源</th>
                    <th>重置时间</th>
                    <th>旧流量</th>
                    <th>新流量</th>
                  </tr>
                </thead>
                <tbody>
                  ${logs.length ? logs.map((log) => `
                    <tr>
                      <td>${log.id}</td>
                      <td>${escapeHtml(log.user_email || "-")}</td>
                      <td>${escapeHtml(log.reset_type_name || log.reset_type || "-")}</td>
                      <td>${escapeHtml(log.trigger_source_name || log.trigger_source || "-")}</td>
                      <td>${escapeHtml(log.reset_time || "-")}</td>
                      <td>${escapeHtml((log.old_traffic && log.old_traffic.formatted) || "-")}</td>
                      <td>${escapeHtml((log.new_traffic && log.new_traffic.formatted) || "-")}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无重置日志</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <pre id="opsTrafficResult">等待操作...</pre>
        `;
      }

      function renderOpsModule(moduleKey) {
        const controller = MODULE_CONTROLLERS[moduleKey];
        return controller && typeof controller.render === "function"
          ? controller.render()
          : '<p class="empty">未实现模块</p>';
      }

      function bindOpsSecurity() {
        const genBtn = document.getElementById("opsGenerateSecurePathBtn");
        const saveBtn = document.getElementById("opsSecuritySaveBtn");

        if (genBtn) {
          genBtn.addEventListener("click", () => {
            setInputValue("safe_secure_path", randomSecurePath());
            markOpsDirty(document.getElementById("safe_secure_path"));
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                safe_mode_enable: readBoolControl("safe_mode_enable"),
                email_verify: readBoolControl("safe_email_verify"),
                register_mode: readValue("safe_register_mode", "all"),
                email_whitelist_enable: readBoolControl("safe_email_whitelist_enable"),
                email_whitelist_suffix: splitLines(readValue("safe_email_whitelist_suffix")),
	                email_gmail_limit_enable: readBoolControl("safe_email_gmail_limit_enable"),
	                secure_path: readValue("safe_secure_path").trim(),
	                login_token_expire_days: Math.max(0, Math.min(3650, readInt("safe_login_token_expire_days", 365))),
	                register_limit_by_ip_enable: readBoolControl("safe_register_limit_by_ip_enable"),
	                register_limit_count: readInt("safe_register_limit_count", 3),
	                register_limit_expire: readInt("safe_register_limit_expire", 60),
	                password_limit_enable: readBoolControl("safe_password_limit_enable"),
	                password_limit_count: readInt("safe_password_limit_count", 5),
	                password_limit_expire: readInt("safe_password_limit_expire", 60),
	                captcha_enable: readBoolControl("safe_captcha_enable"),
	                captcha_type: readValue("safe_captcha_type", "recaptcha"),
	                recaptcha_key: readValue("safe_recaptcha_key"),
	                recaptcha_site_key: readValue("safe_recaptcha_site_key"),
	                recaptcha_v3_secret_key: readValue("safe_recaptcha_v3_secret_key"),
                recaptcha_v3_site_key: readValue("safe_recaptcha_v3_site_key"),
                recaptcha_v3_score_threshold: readFloat("safe_recap_score", 0.5),
                turnstile_secret_key: readValue("safe_turnstile_secret_key"),
                turnstile_site_key: readValue("safe_turnstile_site_key"),
                pow_enable: readBoolControl("safe_pow_enable"),
                pow_auto_scale_enable: readBoolControl("safe_pow_auto_scale_enable"),
                pow_difficulty: readInt("safe_pow_difficulty", 4),
                pow_auto_max_difficulty: readInt("safe_pow_auto_max_difficulty", 7),
                pow_ttl: readInt("safe_pow_ttl", 120),
                pow_seed_salt: readValue("safe_pow_seed_salt"),
                pow_base_value: readValue("safe_pow_base_value"),
                pow_require_ja3: readBoolControl("safe_pow_require_ja3")
              };

              const securePathChanged = Boolean(payload.secure_path && payload.secure_path !== securePath);
              if (securePathChanged) {
                const decision = await requestActionConfirmation({
                  title: "修改后台入口",
                  message: `保存后当前地址将失效，新入口为 /${payload.secure_path}。`,
                  confirmLabel: "确认修改"
                });
                if (!decision.confirmed) return;
              }

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload, clearsDirty: "module" });
              state.opsCache.config = null;
              setPreOutput("opsSecurityResult", resp);
              showToast("安全设置已保存", "success");
              if (securePathChanged) {
                resetOpsDirtyState();
                window.location.assign(`/${encodeURIComponent(payload.secure_path)}#ops/security`);
                return;
              }
              await openOpsModule("security", { force: true });
            } catch (err) {
              setPreOutput("opsSecurityResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }
      }

      function bindOpsOAuth() {
        const previewBtn = document.getElementById("opsOAuthUsePreviewBtn");
        const saveBtn = document.getElementById("opsOAuthSaveBtn");

        if (previewBtn) {
          previewBtn.addEventListener("click", () => {
            const config = state.opsCache.config || {};
            const site = config.site || {};
            const appUrl = String(normalizeConfigValue(site.app_url, settings.base_url || "")).replace(/\/+$/, "");
            const callbackPreview = appUrl
              ? `${appUrl}/api/v1/passport/oauth2/linux-do/callback`
              : "/api/v1/passport/oauth2/linux-do/callback";
            setInputValue("oauth_linux_do_redirect_uri", callbackPreview);
            markOpsDirty(document.getElementById("oauth_linux_do_redirect_uri"));
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                oauth_linux_do_enable: readBoolControl("oauth_linux_do_enable"),
                oauth_linux_do_client_id: readValue("oauth_linux_do_client_id").trim(),
                oauth_linux_do_client_secret: readValue("oauth_linux_do_client_secret").trim(),
                oauth_linux_do_redirect_uri: readValue("oauth_linux_do_redirect_uri").trim()
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload, clearsDirty: "module" });
              state.opsCache.config = null;
              setPreOutput("opsOAuthResult", resp);
              showToast("第三方登录设置已保存", "success");
              await openOpsModule("oauth", { force: true });
            } catch (err) {
              setPreOutput("opsOAuthResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }
      }

      function bindOpsSite() {
        const saveBtn = document.getElementById("opsSiteSaveBtn");
        if (!saveBtn) return;
        saveBtn.addEventListener("click", async () => {
          try {
            const payload = {
              app_name: readValue("site_app_name"),
              app_description: readValue("site_app_description"),
              app_url: readValue("site_app_url"),
              subscribe_url: readValue("site_subscribe_url"),
              subscribe_root_domains: readValue("site_subscribe_root_domains"),
              logo: readValue("site_logo"),
              tos_url: readValue("site_tos_url"),
              force_https: readBoolControl("site_force_https"),
              currency: readValue("site_currency"),
              currency_symbol: readValue("site_currency_symbol"),
              try_out_plan_id: readInt("site_try_out_plan_id", 0),
              try_out_hour: readInt("site_try_out_hour", 1),
              frontend_theme_sidebar: readValue("site_frontend_sidebar"),
              frontend_theme_header: readValue("site_frontend_header"),
              frontend_theme_color: readValue("site_frontend_color"),
              frontend_background_url: readValue("site_frontend_background_url"),
              subscribe_path: readValue("site_subscribe_path"),
              plan_change_enable: readBoolControl("site_plan_change_enable"),
              surplus_enable: readBoolControl("site_surplus_enable")
            };
            const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload, clearsDirty: "module" });
            state.opsCache.config = null;
            setPreOutput("opsSiteResult", resp);
          showToast("站点设置已保存", "success");
          await openOpsModule("site", { force: true });
        } catch (err) {
          setPreOutput("opsSiteResult", err.message || "保存失败");
          showToast(err.message || "保存失败", "error");
        }
      });
    }

      function bindOpsTelegram() {
        const saveBtn = document.getElementById("opsTelegramSaveBtn");
        const webhookBtn = document.getElementById("opsRegisterTelegramWebhookBtn");

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                telegram_bot_enable: readBoolControl("telegram_bot_enable"),
                telegram_bot_token: readValue("telegram_bot_token").trim(),
                telegram_discuss_link: readValue("telegram_discuss_link"),
                telegram_user_ticket_enable: readBoolControl("telegram_user_ticket_enable"),
                telegram_notify_ticket_created: readBoolControl("telegram_notify_ticket_created"),
                telegram_notify_ticket_replied: readBoolControl("telegram_notify_ticket_replied"),
                telegram_notify_ticket_closed: readBoolControl("telegram_notify_ticket_closed"),
                telegram_notify_payment_success: readBoolControl("telegram_notify_payment_success"),
                telegram_notify_notice_published: readBoolControl("telegram_notify_notice_published"),
                telegram_notify_tcping_alert: readBoolControl("telegram_notify_tcping_alert"),
                telegram_notify_tcping_recover: readBoolControl("telegram_notify_tcping_recover"),
                telegram_notify_refund_vote: readBoolControl("telegram_notify_refund_vote"),
                telegram_notify_refund_status: readBoolControl("telegram_notify_refund_status"),
                telegram_notify_user_risk_detected: readBoolControl("telegram_notify_user_risk_detected"),
                telegram_notify_user_banned: readBoolControl("telegram_notify_user_banned")
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload, clearsDirty: "module" });
              state.opsCache.config = null;
              setPreOutput("opsTelegramResult", resp);
              showToast("Telegram 设置已保存", "success");
              await openOpsModule("telegram", { force: true });
            } catch (err) {
              setPreOutput("opsTelegramResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (webhookBtn) {
          webhookBtn.addEventListener("click", async () => {
            try {
              const resp = await request({ method: "POST", url: buildV2("config/setTelegramWebhook") });
              setPreOutput("opsTelegramResult", resp);
              showToast("Webhook 已注册/刷新", "success");
            } catch (err) {
              setPreOutput("opsTelegramResult", err.message || "Webhook 注册失败");
              showToast(err.message || "Webhook 注册失败", "error");
            }
          });
        }
      }

      function bindOpsRiskReview() {
        const saveBtn = document.getElementById("opsRiskReviewSaveBtn");
        const runBtn = document.getElementById("opsRiskReviewRunBtn");
        const reloadBtn = document.getElementById("opsRiskReviewReloadBtn");
        const prevBtn = document.getElementById("opsRiskReviewPrevBtn");
        const nextBtn = document.getElementById("opsRiskReviewNextBtn");
        const banPrevBtn = document.getElementById("opsBanRecordPrevBtn");
        const banNextBtn = document.getElementById("opsBanRecordNextBtn");
        const notificationsBtn = document.getElementById("opsRiskNotificationsBtn");
        const banButtons = Array.from(document.querySelectorAll(".risk-review-ban-btn"));

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                user_risk_review_enable: readBoolControl("risk_user_risk_review_enable"),
                user_risk_review_schedule_minutes: readInt("risk_user_risk_review_schedule_minutes", 30),
                user_risk_review_time_window_minutes: readInt("risk_user_risk_review_time_window_minutes", 60),
                user_risk_review_context_hours: readInt("risk_user_risk_review_context_hours", 24),
                user_risk_review_min_shared_ip_users: readInt("risk_user_risk_review_min_shared_ip_users", 2),
                user_risk_review_scan_limit: readInt("risk_user_risk_review_scan_limit", 20),
                user_risk_review_notify_cooldown_minutes: readInt("risk_user_risk_review_notify_cooldown_minutes", 60),
                user_risk_review_llm_enable: readBoolControl("risk_user_risk_review_llm_enable"),
                user_risk_review_llm_base_url: readValue("risk_user_risk_review_llm_base_url").trim(),
                user_risk_review_llm_api_key: readValue("risk_user_risk_review_llm_api_key").trim(),
                user_risk_review_llm_model: readValue("risk_user_risk_review_llm_model").trim(),
                user_risk_review_llm_timeout_seconds: readInt("risk_user_risk_review_llm_timeout_seconds", 20),
                user_risk_review_llm_temperature: readFloat("risk_user_risk_review_llm_temperature", 0.2),
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload, clearsDirty: "module" });
              state.opsCache.config = null;
              setPreOutput("opsRiskReviewResult", resp);
              showToast("风险审查设置已保存", "success");
              await openOpsModule("riskreview", { force: true });
            } catch (err) {
              setPreOutput("opsRiskReviewResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (runBtn) {
          runBtn.addEventListener("click", async () => {
            try {
              const resp = await request({ method: "POST", url: buildV2("risk-review/run"), data: { limit: readInt("risk_user_risk_review_scan_limit", 20) } });
              setPreOutput("opsRiskReviewResult", resp);
              showToast("风险审查已执行", "success");
              await openOpsModule("riskreview", { force: true });
            } catch (err) {
              setPreOutput("opsRiskReviewResult", err.message || "执行失败");
              showToast(err.message || "执行失败", "error");
            }
          });
        }

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("riskreview", { force: true }));
        }

        if (notificationsBtn) {
          notificationsBtn.addEventListener("click", () => {
            activateView("ops", "telegram", { syncRoute: true }).catch((err) => {
              showToast(err.message || "通知设置加载失败", "error");
            });
          });
        }

        if (prevBtn) {
          prevBtn.addEventListener("click", async () => {
            state.opsCache.riskReviewPage = Math.max(1, Number(state.opsCache.riskReviewPage || 1) - 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        if (nextBtn) {
          nextBtn.addEventListener("click", async () => {
            const lastPage = Number((state.opsCache.riskReviewPagination || {}).last_page || 1);
            state.opsCache.riskReviewPage = Math.min(lastPage, Number(state.opsCache.riskReviewPage || 1) + 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        if (banPrevBtn) {
          banPrevBtn.addEventListener("click", async () => {
            state.opsCache.banRecordPage = Math.max(1, Number(state.opsCache.banRecordPage || 1) - 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        if (banNextBtn) {
          banNextBtn.addEventListener("click", async () => {
            const lastPage = Number((state.opsCache.banRecordPagination || {}).last_page || 1);
            state.opsCache.banRecordPage = Math.min(lastPage, Number(state.opsCache.banRecordPage || 1) + 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        banButtons.forEach((btn) => {
          btn.addEventListener("click", async () => {
            try {
              const userId = Number(btn.dataset.userId || 0);
              const userEmail = String(btn.dataset.userEmail || "");
              const defaultReason = String(btn.dataset.banReason || "");
              if (!userId) throw new Error("缺少用户编号");
              const decision = await requestActionConfirmation({
                title: "封禁风险用户",
                message: `将立即阻止 ${userEmail || `#${userId}`} 登录和使用服务。`,
                confirmLabel: "确认封禁",
                showReason: true,
                requireReason: true,
                defaultReason,
                reasonPlaceholder: "填写封禁原因"
              });
              if (!decision.confirmed) return;
              await request({
                method: "POST",
                url: buildV2("user/update"),
                data: {
                  id: userId,
                  banned: 1,
                  ban_reason: decision.reason,
                }
              });
              showToast("用户已封禁", "success");
              await loadUsers().catch(() => {});
              await openOpsModule("riskreview", { force: true });
            } catch (err) {
              showToast(err.message || "封禁失败", "error");
            }
          });
        });
      }

      function bindOpsPlans() {
        const plans = state.opsCache.plans || [];
        const saveBtn = document.getElementById("opsPlanSaveBtn");
        const clearBtn = document.getElementById("opsPlanClearBtn");
        const reloadBtn = document.getElementById("opsPlansReloadBtn");
        const tableBody = document.getElementById("opsPlansTableBody");

        const clearForm = () => {
          setInputValue("opsPlanId", "");
          setInputValue("opsPlanName", "");
          setInputValue("opsPlanGroupId", "");
          setInputValue("opsPlanTransfer", "100");
          setInputValue("opsPlanSpeedLimit", "0");
          setInputValue("opsPlanDeviceLimit", "0");
          setInputValue("opsPlanCapacityLimit", "0");
          setInputValue("opsPlanResetMethod", "0");
          setInputValue("opsPlanTags", "");
          setInputValue("opsPlanContent", "");
          setInputValue("opsPlanPrices", "{}");
        };

        const fillFromPlan = (plan) => {
          if (!plan) return;
          setInputValue("opsPlanId", plan.id);
          setInputValue("opsPlanName", plan.name || "");
          setInputValue("opsPlanGroupId", plan.group_id || "");
          setInputValue("opsPlanTransfer", plan.transfer_enable || 100);
          setInputValue("opsPlanSpeedLimit", plan.speed_limit || 0);
          setInputValue("opsPlanDeviceLimit", plan.device_limit || 0);
          setInputValue("opsPlanCapacityLimit", plan.capacity_limit || 0);
          setInputValue("opsPlanResetMethod", plan.reset_traffic_method || 0);
          setInputValue("opsPlanTags", Array.isArray(plan.tags) ? plan.tags.join(",") : "");
          setInputValue("opsPlanContent", plan.content || "");

          let prices = {};
          if (plan.prices && typeof plan.prices === "object") {
            prices = plan.prices;
          } else {
            const fallbackKeys = ["month_price", "quarter_price", "half_year_price", "year_price", "two_year_price", "three_year_price", "onetime_price", "reset_price"];
            fallbackKeys.forEach((key) => {
              if (plan[key] != null) prices[key] = plan[key];
            });
          }
          setInputValue("opsPlanPrices", JSON.stringify(prices, null, 2));
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("plans", { force: true }));
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", () => {
            clearForm();
            markOpsDirty(document.getElementById("opsPlanName"));
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsPlanId", 0, true);
              const payload = {
                name: readValue("opsPlanName").trim(),
                transfer_enable: readInt("opsPlanTransfer", 100),
                group_id: readInt("opsPlanGroupId", 0, true),
                speed_limit: readInt("opsPlanSpeedLimit", 0, true),
                device_limit: readInt("opsPlanDeviceLimit", 0, true),
                capacity_limit: readInt("opsPlanCapacityLimit", 0, true),
                reset_traffic_method: readInt("opsPlanResetMethod", 0, true),
                tags: splitCsv(readValue("opsPlanTags")),
                content: readValue("opsPlanContent"),
                prices: parseJsonInput("opsPlanPrices", {})
              };
              if (!payload.name) {
                throw new Error("套餐名称不能为空");
              }
              if (id) payload.id = id;
              const resp = await request({ method: "POST", url: buildV2("plan/save"), data: payload, clearsDirty: true });
              setPreOutput("opsPlanResult", resp);
              showToast("套餐已保存", "success");
              await openOpsModule("plans", { force: true });
            } catch (err) {
              setPreOutput("opsPlanResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-plan-action]");
            if (!btn) return;
            const action = btn.dataset.planAction;
            const id = Number(btn.dataset.id);
            const plan = plans.find((item) => Number(item.id) === id);
            if (!id || !plan) return;

            try {
              if (action === "edit") {
                fillFromPlan(plan);
                showToast(`已载入套餐 ${id}`, "info");
                return;
              }

              if (action === "drop") {
                const decision = await requestActionConfirmation({
                  title: "删除套餐",
                  message: `套餐 #${id} 将被永久删除。`,
                  confirmLabel: "确认删除"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("plan/drop"), data: { id } });
                setPreOutput("opsPlanResult", resp);
                showToast("套餐已删除", "success");
                await openOpsModule("plans", { force: true });
                return;
              }

              if (action === "toggle-show" || action === "toggle-sell") {
                const payload = {
                  id,
                  show: Number(plan.show) || 0,
                  sell: Number(plan.sell) || 0,
                  renew: Number(plan.renew) || 0
                };
                if (action === "toggle-show") payload.show = payload.show ? 0 : 1;
                if (action === "toggle-sell") payload.sell = payload.sell ? 0 : 1;
                const resp = await request({ method: "POST", url: buildV2("plan/update"), data: payload });
                setPreOutput("opsPlanResult", resp);
                showToast("套餐状态已更新", "success");
                await openOpsModule("plans", { force: true });
              }
            } catch (err) {
              setPreOutput("opsPlanResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsPayments() {
        const payments = state.opsCache.payments || [];
        const policySaveBtn = document.getElementById("opsPaymentPolicySaveBtn");
        const saveBtn = document.getElementById("opsPaymentSaveBtn");
        const clearBtn = document.getElementById("opsPaymentClearBtn");
        const reloadBtn = document.getElementById("opsPaymentsReloadBtn");
        const tableBody = document.getElementById("opsPaymentsTableBody");

        const clearForm = () => {
          setInputValue("opsPaymentId", "");
          setInputValue("opsPaymentName", "");
          setInputValue("opsPaymentGateway", "");
          setInputValue("opsPaymentIcon", "");
          setInputValue("opsPaymentNotifyDomain", "");
          setInputValue("opsPaymentFeeFixed", "0");
          setInputValue("opsPaymentFeePercent", "0");
          setInputValue("opsPaymentConfig", "{}");
        };

        const fillFromPayment = (payment) => {
          if (!payment) return;
          setInputValue("opsPaymentId", payment.id);
          setInputValue("opsPaymentName", payment.name || "");
          setInputValue("opsPaymentGateway", payment.payment || "");
          setInputValue("opsPaymentIcon", payment.icon || "");
          setInputValue("opsPaymentNotifyDomain", payment.notify_domain || "");
          setInputValue("opsPaymentFeeFixed", payment.handling_fee_fixed || 0);
          setInputValue("opsPaymentFeePercent", payment.handling_fee_percent || 0);
          const config = typeof payment.config === "object"
            ? payment.config
            : parseJsonText(payment.config, {});
          setInputValue("opsPaymentConfig", JSON.stringify(config, null, 2));
        };

        if (policySaveBtn) {
          policySaveBtn.addEventListener("click", async () => {
            try {
              const resp = await request({
                method: "POST",
                url: buildV2("config/save"),
                data: { refund_dispute_enable: readBoolControl("opsRefundDisputeEnable") },
                clearsDirty: true
              });
              state.opsCache.config = null;
              setPreOutput("opsPaymentResult", resp);
              showToast("退款争议策略已保存", "success");
              await openOpsModule("payments", { force: true });
            } catch (err) {
              setPreOutput("opsPaymentResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("payments", { force: true }));
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", () => {
            clearForm();
            markOpsDirty(document.getElementById("opsPaymentName"));
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsPaymentId", 0, true);
              const payload = {
                name: readValue("opsPaymentName").trim(),
                payment: readValue("opsPaymentGateway").trim(),
                icon: readValue("opsPaymentIcon").trim(),
                config: parseJsonInput("opsPaymentConfig", {}),
                notify_domain: readValue("opsPaymentNotifyDomain").trim() || null,
                handling_fee_fixed: readInt("opsPaymentFeeFixed", 0, true),
                handling_fee_percent: readFloat("opsPaymentFeePercent", 0, true)
              };
              if (!payload.name || !payload.payment) {
                throw new Error("名称和支付方式标识不能为空");
              }
              if (id) payload.id = id;
              const resp = await request({ method: "POST", url: buildV2("payment/save"), data: payload, clearsDirty: true });
              setPreOutput("opsPaymentResult", resp);
              showToast("支付方式已保存", "success");
              await openOpsModule("payments", { force: true });
            } catch (err) {
              setPreOutput("opsPaymentResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-payment-action]");
            if (!btn) return;
            const action = btn.dataset.paymentAction;
            const id = Number(btn.dataset.id);
            const payment = payments.find((item) => Number(item.id) === id);
            if (!id || !payment) return;
            try {
              if (action === "edit") {
                fillFromPayment(payment);
                showToast(`已载入支付方式 #${id}`, "info");
                return;
              }
              if (action === "toggle") {
                const resp = await request({ method: "POST", url: buildV2("payment/show"), data: { id } });
                setPreOutput("opsPaymentResult", resp);
                showToast("支付方式状态已切换", "success");
                await openOpsModule("payments", { force: true });
                return;
              }
              if (action === "drop") {
                const decision = await requestActionConfirmation({
                  title: "删除支付渠道",
                  message: `支付渠道 #${id} 将被永久删除。`,
                  confirmLabel: "确认删除"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("payment/drop"), data: { id } });
                setPreOutput("opsPaymentResult", resp);
                showToast("支付方式已删除", "success");
                await openOpsModule("payments", { force: true });
              }
            } catch (err) {
              setPreOutput("opsPaymentResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsNotices() {
        const notices = state.opsCache.notices || [];
        const saveBtn = document.getElementById("opsNoticeSaveBtn");
        const clearBtn = document.getElementById("opsNoticeClearBtn");
        const reloadBtn = document.getElementById("opsNoticesReloadBtn");
        const tableBody = document.getElementById("opsNoticesTableBody");

        const clearForm = () => {
          setInputValue("opsNoticeId", "");
          setInputValue("opsNoticeTitle", "");
          setInputValue("opsNoticeContent", "");
          setInputValue("opsNoticeImgUrl", "");
          setInputValue("opsNoticeTags", "");
          setInputValue("opsNoticeShow", "1");
          setInputValue("opsNoticePopup", "0");
        };

        const fillForm = (notice) => {
          if (!notice) return;
          setInputValue("opsNoticeId", notice.id);
          setInputValue("opsNoticeTitle", notice.title || "");
          setInputValue("opsNoticeContent", notice.content || "");
          setInputValue("opsNoticeImgUrl", notice.img_url || "");
          setInputValue("opsNoticeTags", Array.isArray(notice.tags) ? notice.tags.join(",") : "");
          setInputValue("opsNoticeShow", isOn(notice.show) ? "1" : "0");
          setInputValue("opsNoticePopup", isOn(notice.popup) ? "1" : "0");
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("notices", { force: true }));
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", () => {
            clearForm();
            markOpsDirty(document.getElementById("opsNoticeTitle"));
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsNoticeId", 0, true);
              const payload = {
                title: readValue("opsNoticeTitle").trim(),
                content: readValue("opsNoticeContent").trim(),
                img_url: readValue("opsNoticeImgUrl").trim() || null,
                tags: splitCsv(readValue("opsNoticeTags")),
                show: readBoolControl("opsNoticeShow"),
                popup: readBoolControl("opsNoticePopup")
              };
              if (!payload.title || !payload.content) {
                throw new Error("标题和内容不能为空");
              }
              if (id) payload.id = id;
              const resp = await request({ method: "POST", url: buildV2("notice/save"), data: payload, clearsDirty: true });
              setPreOutput("opsNoticeResult", resp);
              showToast("公告已保存", "success");
              await openOpsModule("notices", { force: true });
            } catch (err) {
              setPreOutput("opsNoticeResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-notice-action]");
            if (!btn) return;
            const action = btn.dataset.noticeAction;
            const id = Number(btn.dataset.id);
            const notice = notices.find((n) => Number(n.id) === id);
            if (!id || !notice) return;

            try {
              if (action === "edit") {
                fillForm(notice);
                return;
              }
              if (action === "toggle") {
                const resp = await request({ method: "POST", url: buildV2("notice/show"), data: { id } });
                setPreOutput("opsNoticeResult", resp);
                showToast("公告显示状态已切换", "success");
                await openOpsModule("notices", { force: true });
                return;
              }
              if (action === "drop") {
                const decision = await requestActionConfirmation({
                  title: "删除公告",
                  message: `公告 #${id} 将被永久删除。`,
                  confirmLabel: "确认删除"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("notice/drop"), data: { id } });
                setPreOutput("opsNoticeResult", resp);
                showToast("公告已删除", "success");
                await openOpsModule("notices", { force: true });
              }
            } catch (err) {
              setPreOutput("opsNoticeResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsTickets() {
        const reloadBtn = document.getElementById("opsTicketsReloadBtn");
        const tableBody = document.getElementById("opsTicketsTableBody");
        const replyBtn = document.getElementById("opsTicketReplyBtn");
        const closeBtn = document.getElementById("opsTicketCloseBtn");
        const detailBtn = document.getElementById("opsTicketDetailBtn");

        const fetchDetail = async () => {
          const id = readInt("opsTicketId", 0);
          if (!id) throw new Error("请输入正确的工单编号");
          const detail = await request({ method: "GET", url: `${buildV2("ticket/fetch")}?id=${id}` });
          setPreOutput("opsTicketDetail", detail);
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("tickets", { force: true }));
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-ticket-action]");
            if (!btn) return;
            const action = btn.dataset.ticketAction;
            const id = Number(btn.dataset.id);
            if (!id) return;

            try {
              if (action === "detail") {
                setInputValue("opsTicketId", id);
                await fetchDetail();
                return;
              }
              if (action === "reply") {
                setInputValue("opsTicketId", id);
                showToast(`已选择工单 #${id}`, "info");
                return;
              }
              if (action === "close") {
                const decision = await requestActionConfirmation({
                  title: "关闭工单",
                  message: `工单 #${id} 将结束处理。`,
                  confirmLabel: "确认关闭"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("ticket/close"), data: { id } });
                setPreOutput("opsTicketResult", resp);
                showToast("工单已关闭", "success");
                await openOpsModule("tickets", { force: true });
              }
            } catch (err) {
              setPreOutput("opsTicketResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }

        if (replyBtn) {
          replyBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsTicketId", 0);
              const message = readValue("opsTicketReply").trim();
              if (!id || !message) {
                throw new Error("工单编号和回复内容不能为空");
              }
              const resp = await request({ method: "POST", url: buildV2("ticket/reply"), data: { id, message }, clearsDirty: true });
              setPreOutput("opsTicketResult", resp);
              showToast("工单回复已发送", "success");
              await openOpsModule("tickets", { force: true });
            } catch (err) {
              setPreOutput("opsTicketResult", err.message || "回复失败");
              showToast(err.message || "回复失败", "error");
            }
          });
        }

        if (closeBtn) {
          closeBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsTicketId", 0);
              if (!id) throw new Error("请输入正确的工单编号");
              const decision = await requestActionConfirmation({
                title: "关闭工单",
                message: `工单 #${id} 将结束处理。`,
                confirmLabel: "确认关闭"
              });
              if (!decision.confirmed) return;
              const resp = await request({ method: "POST", url: buildV2("ticket/close"), data: { id } });
              setPreOutput("opsTicketResult", resp);
              showToast("工单已关闭", "success");
              await openOpsModule("tickets", { force: true });
            } catch (err) {
              setPreOutput("opsTicketResult", err.message || "关闭失败");
              showToast(err.message || "关闭失败", "error");
            }
          });
        }

        if (detailBtn) {
          detailBtn.addEventListener("click", async () => {
            try {
              await fetchDetail();
            } catch (err) {
              setPreOutput("opsTicketDetail", err.message || "读取失败");
              showToast(err.message || "读取失败", "error");
            }
          });
        }
      }

      function bindOpsCoupons() {
        const reloadBtn = document.getElementById("opsCouponsReloadBtn");
        const generateBtn = document.getElementById("opsCouponGenerateBtn");
        const tableBody = document.getElementById("opsCouponsTableBody");

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("coupons", { force: true }));
        }

        if (generateBtn) {
          generateBtn.addEventListener("click", async () => {
            try {
              const payload = {
                generate_count: readInt("opsCouponGenerateCount", 1),
                name: readValue("opsCouponName").trim(),
                type: readValue("opsCouponType"),
                value: readInt("opsCouponValue", 0),
                started_at: toUnixFromInput(readValue("opsCouponStartedAt")),
                ended_at: toUnixFromInput(readValue("opsCouponEndedAt")),
                limit_use: readInt("opsCouponLimitUse", 0, true),
                limit_use_with_user: readInt("opsCouponLimitUseWithUser", 0, true),
                limit_plan_ids: splitCsv(readValue("opsCouponLimitPlanIds")).map((v) => Number(v)).filter((v) => Number.isFinite(v) && v > 0),
                limit_period: splitCsv(readValue("opsCouponLimitPeriod"))
              };
              const fixedCode = readValue("opsCouponCode").trim();
              if (fixedCode) payload.code = fixedCode;
              if (!payload.name || !payload.type || !payload.value || !payload.started_at || !payload.ended_at) {
                throw new Error("请完整填写名称、类型、值、开始和结束时间");
              }
              const resp = await request({ method: "POST", url: buildV2("coupon/generate"), data: payload, clearsDirty: true });
              setPreOutput("opsCouponResult", resp);
              showToast("优惠券生成成功", "success");
              await openOpsModule("coupons", { force: true });
            } catch (err) {
              setPreOutput("opsCouponResult", err.message || "生成失败");
              showToast(err.message || "生成失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-coupon-action]");
            if (!btn) return;
            const action = btn.dataset.couponAction;
            const id = Number(btn.dataset.id);
            if (!id) return;

            try {
              if (action === "toggle") {
                const resp = await request({ method: "POST", url: buildV2("coupon/show"), data: { id } });
                setPreOutput("opsCouponResult", resp);
                showToast("优惠券显示状态已更新", "success");
                await openOpsModule("coupons", { force: true });
                return;
              }
              if (action === "drop") {
                const decision = await requestActionConfirmation({
                  title: "删除优惠券",
                  message: `优惠券 #${id} 将被永久删除。`,
                  confirmLabel: "确认删除"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("coupon/drop"), data: { id } });
                setPreOutput("opsCouponResult", resp);
                showToast("优惠券已删除", "success");
                await openOpsModule("coupons", { force: true });
              }
            } catch (err) {
              setPreOutput("opsCouponResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsGiftCards() {
        const templates = state.opsCache.giftTemplates || [];
        const saveTemplateBtn = document.getElementById("opsGiftTemplateSaveBtn");
        const clearTemplateBtn = document.getElementById("opsGiftTemplateClearBtn");
        const generateCodeBtn = document.getElementById("opsGiftCodeGenerateBtn");
        const reloadBtn = document.getElementById("opsGiftReloadBtn");
        const templateBody = document.getElementById("opsGiftTemplateTableBody");
        const codeBody = document.getElementById("opsGiftCodeTableBody");

        const clearTemplateForm = () => {
          setInputValue("opsGiftTemplateId", "");
          setInputValue("opsGiftTemplateName", "");
          setInputValue("opsGiftTemplateDesc", "");
          setInputValue("opsGiftTemplateThemeColor", "#1890ff");
          setInputValue("opsGiftTemplateRewards", '{"balance":1000}');
          setInputValue("opsGiftTemplateConditions", "{}");
          setInputValue("opsGiftTemplateLimits", "{}");
          setInputValue("opsGiftTemplateStatus", "1");
        };

        const fillTemplateForm = (template) => {
          if (!template) return;
          setInputValue("opsGiftTemplateId", template.id);
          setInputValue("opsGiftTemplateName", template.name || "");
          setInputValue("opsGiftTemplateType", template.type || "");
          setInputValue("opsGiftTemplateStatus", isOn(template.status) ? "1" : "0");
          setInputValue("opsGiftTemplateDesc", template.description || "");
          setInputValue("opsGiftTemplateThemeColor", template.theme_color || "#1890ff");
          setInputValue("opsGiftTemplateRewards", JSON.stringify(template.rewards || {}, null, 2));
          setInputValue("opsGiftTemplateConditions", JSON.stringify(template.conditions || {}, null, 2));
          setInputValue("opsGiftTemplateLimits", JSON.stringify(template.limits || {}, null, 2));
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("giftcards", { force: true }));
        }

        if (clearTemplateBtn) {
          clearTemplateBtn.addEventListener("click", () => {
            clearTemplateForm();
            markOpsDirty(document.getElementById("opsGiftTemplateName"));
          });
        }

        if (saveTemplateBtn) {
          saveTemplateBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsGiftTemplateId", 0, true);
              const payload = {
                name: readValue("opsGiftTemplateName").trim(),
                type: readInt("opsGiftTemplateType", 1),
                status: readBoolControl("opsGiftTemplateStatus"),
                description: readValue("opsGiftTemplateDesc"),
                theme_color: readValue("opsGiftTemplateThemeColor") || "#1890ff",
                rewards: parseJsonInput("opsGiftTemplateRewards", {}),
                conditions: parseJsonInput("opsGiftTemplateConditions", {}),
                limits: parseJsonInput("opsGiftTemplateLimits", {})
              };
              if (!payload.name) throw new Error("模板名称不能为空");
              let resp;
              if (id) {
                payload.id = id;
                resp = await request({ method: "POST", url: buildV2("gift-card/update-template"), data: payload, clearsDirty: true });
              } else {
                resp = await request({ method: "POST", url: buildV2("gift-card/create-template"), data: payload, clearsDirty: true });
              }
              setPreOutput("opsGiftResult", resp);
              showToast("礼品卡模板已保存", "success");
              await openOpsModule("giftcards", { force: true });
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (generateCodeBtn) {
          generateCodeBtn.addEventListener("click", async () => {
            try {
              const payload = {
                template_id: readInt("opsGiftCodeTemplateId", 0),
                count: readInt("opsGiftCodeCount", 10),
                prefix: readValue("opsGiftCodePrefix", "GC").trim() || "GC",
                max_usage: readInt("opsGiftCodeMaxUsage", 1)
              };
              const expiresHours = readInt("opsGiftCodeHours", 0, true);
              if (expiresHours) payload.expires_hours = expiresHours;
              if (!payload.template_id || !payload.count) throw new Error("模板编号和数量不能为空");
              const resp = await request({ method: "POST", url: buildV2("gift-card/generate-codes"), data: payload, clearsDirty: true });
              setPreOutput("opsGiftResult", resp);
              showToast("兑换码生成成功", "success");
              await openOpsModule("giftcards", { force: true });
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "生成失败");
              showToast(err.message || "生成失败", "error");
            }
          });
        }

        if (templateBody) {
          templateBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-gift-template-action]");
            if (!btn) return;
            const action = btn.dataset.giftTemplateAction;
            const id = Number(btn.dataset.id);
            const template = templates.find((t) => Number(t.id) === id);
            if (!id || !template) return;
            try {
              if (action === "edit") {
                fillTemplateForm(template);
                return;
              }
              if (action === "drop") {
                const decision = await requestActionConfirmation({
                  title: "删除礼品卡模板",
                  message: `模板 #${id} 将被永久删除。`,
                  confirmLabel: "确认删除"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("gift-card/delete-template"), data: { id } });
                setPreOutput("opsGiftResult", resp);
                showToast("模板已删除", "success");
                await openOpsModule("giftcards", { force: true });
              }
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }

        if (codeBody) {
          codeBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-gift-code-action]");
            if (!btn) return;
            const action = btn.dataset.giftCodeAction;
            const id = Number(btn.dataset.id);
            if (!id) return;
            try {
              if (action === "toggle") {
                const currentStatus = Number(btn.dataset.status);
                const toggleAction = currentStatus === 3 ? "enable" : "disable";
                const resp = await request({ method: "POST", url: buildV2("gift-card/toggle-code"), data: { id, action: toggleAction } });
                setPreOutput("opsGiftResult", resp);
                showToast("兑换码状态已更新", "success");
                await openOpsModule("giftcards", { force: true });
                return;
              }
              if (action === "drop") {
                const decision = await requestActionConfirmation({
                  title: "删除兑换码",
                  message: `兑换码 #${id} 将被永久删除。`,
                  confirmLabel: "确认删除"
                });
                if (!decision.confirmed) return;
                const resp = await request({ method: "POST", url: buildV2("gift-card/delete-code"), data: { id } });
                setPreOutput("opsGiftResult", resp);
                showToast("兑换码已删除", "success");
                await openOpsModule("giftcards", { force: true });
              }
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsPlugins() {
        const plugins = state.opsCache.plugins || [];
        const reloadBtn = document.getElementById("opsPluginsReloadBtn");
        const tableBody = document.getElementById("opsPluginsTableBody");
        const saveConfigBtn = document.getElementById("opsPluginSaveConfigBtn");

        const actionEndpointMap = {
          install: "plugin/install",
          uninstall: "plugin/uninstall",
          enable: "plugin/enable",
          disable: "plugin/disable",
          upgrade: "plugin/upgrade",
          delete: "plugin/delete"
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("plugins", { force: true }));
        }

        if (saveConfigBtn) {
          saveConfigBtn.addEventListener("click", async () => {
            try {
              const code = readValue("opsPluginCode").trim();
              if (!code) throw new Error("请输入扩展标识");
              const config = parseJsonInput("opsPluginConfig", {});
              const resp = await request({ method: "POST", url: buildV2("plugin/config"), data: { code, config }, clearsDirty: true });
              setPreOutput("opsPluginResult", resp);
              showToast("扩展设置已保存", "success");
              await openOpsModule("plugins", { force: true });
            } catch (err) {
              setPreOutput("opsPluginResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-plugin-action]");
            if (!btn) return;
            const action = btn.dataset.pluginAction;
            const code = btn.dataset.code;
            if (!code) return;

            try {
              if (action === "config") {
                const plugin = plugins.find((p) => String(p.code) === String(code));
                setInputValue("opsPluginCode", code);
                setInputValue("opsPluginConfig", JSON.stringify((plugin && plugin.config) || {}, null, 2));
                showToast(`已加载 ${code} 配置`, "info");
                return;
              }

              const endpoint = actionEndpointMap[action];
              if (!endpoint) return;
              if (action === "uninstall" || action === "delete") {
                const decision = await requestActionConfirmation({
                  title: action === "delete" ? "删除扩展" : "卸载扩展",
                  message: `${code} 的${action === "delete" ? "文件与记录" : "安装状态"}将被移除。`,
                  confirmLabel: action === "delete" ? "确认删除" : "确认卸载"
                });
                if (!decision.confirmed) return;
              }
              const resp = await request({ method: "POST", url: buildV2(endpoint), data: { code } });
              setPreOutput("opsPluginResult", resp);
              showToast(`扩展 ${code} 操作成功`, "success");
              await openOpsModule("plugins", { force: true });
            } catch (err) {
              setPreOutput("opsPluginResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsSystem() {
        const reloadBtn = document.getElementById("opsSystemReloadBtn");
        const loadLogsBtn = document.getElementById("opsSystemLoadLogsBtn");
        const clearBtn = document.getElementById("opsSystemClearBtn");

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("system", { force: true }));
        }

        if (loadLogsBtn) {
          loadLogsBtn.addEventListener("click", async () => {
            try {
              const requestEpoch = state.navigationEpoch;
              const level = readValue("opsSystemLogLevel");
              const keyword = readValue("opsSystemLogKeyword").trim();
              const clearInput = document.getElementById("opsSystemClearDays");
              const clearScope = clearInput?.closest?.(".ops-card") || null;
              const query = new URLSearchParams({ current: "1", page_size: "20" });
              if (level) query.set("level", level);
              if (keyword) query.set("keyword", keyword);
              const body = await request({ method: "GET", url: `${buildV2("system/getSystemLog")}?${query.toString()}` });
              state.opsCache.systemLogs = normalizeObject(body);
              if (requestEpoch !== state.navigationEpoch || state.activeTab !== "ops" || state.activeOpsModule !== "system") {
                return;
              }
              const retainedLevel = readValue("opsSystemLogLevel", level);
              const retainedKeyword = readValue("opsSystemLogKeyword", keyword);
              const retainedClearDraft = {
                days: readValue("opsSystemClearDays", "30"),
                level: readValue("opsSystemClearLevel", "all"),
                limit: readValue("opsSystemClearLimit", "1000")
              };
              const clearDraftWasDirty = Boolean(clearScope && state.opsDirtyScopes.has(clearScope));
              dom.opsWorkspace.innerHTML = renderOpsSystem();
              enhanceWorkspaceTables(dom.opsWorkspace, OPS_MODULE_MAP.system.name);
              resetOpsDirtyState();
              setInputValue("opsSystemLogLevel", retainedLevel);
              setInputValue("opsSystemLogKeyword", retainedKeyword);
              setInputValue("opsSystemClearDays", retainedClearDraft.days);
              setInputValue("opsSystemClearLevel", retainedClearDraft.level);
              setInputValue("opsSystemClearLimit", retainedClearDraft.limit);
              if (clearDraftWasDirty) {
                markOpsDirty(document.getElementById("opsSystemClearDays"));
              }
              bindOpsSystem();
            } catch (err) {
              setPreOutput("opsSystemResult", err.message || "读取失败");
              showToast(err.message || "读取失败", "error");
            }
          });
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", async () => {
            try {
              const payload = {
                days: readInt("opsSystemClearDays", 30),
                level: readValue("opsSystemClearLevel", "all"),
                limit: readInt("opsSystemClearLimit", 1000)
              };
              const decision = await requestActionConfirmation({
                title: "清理系统日志",
                message: `将按当前条件清理 ${payload.days} 天前的日志，最多处理 ${payload.limit} 条。`,
                confirmLabel: "确认清理"
              });
              if (!decision.confirmed) return;
              const resp = await request({ method: "POST", url: buildV2("system/clearSystemLog"), data: payload, clearsDirty: true });
              setPreOutput("opsSystemResult", resp);
              showToast("日志清理完成", "success");
              await openOpsModule("system", { force: true });
            } catch (err) {
              setPreOutput("opsSystemResult", err.message || "清理失败");
              showToast(err.message || "清理失败", "error");
            }
          });
        }
      }

      function bindOpsTraffic() {
        const reloadBtn = document.getElementById("opsTrafficReloadBtn");
        const resetBtn = document.getElementById("opsTrafficResetBtn");
        const historyBtn = document.getElementById("opsTrafficHistoryBtn");

        if (reloadBtn) {
          reloadBtn.addEventListener("click", async () => {
            state.opsCache.trafficDays = readInt("opsTrafficDays", 30);
            await openOpsModule("traffic", { force: true });
          });
        }

        if (resetBtn) {
          resetBtn.addEventListener("click", async () => {
            try {
              const userId = readInt("opsTrafficUserId", 0);
              const reason = readValue("opsTrafficReason").trim();
              if (!userId) throw new Error("请输入用户编号");
              const decision = await requestActionConfirmation({
                title: "重置用户流量",
                message: `用户 #${userId} 的流量统计将立即重置。`,
                confirmLabel: "确认重置",
                showReason: true,
                requireReason: true,
                defaultReason: reason,
                reasonPlaceholder: "填写重置原因"
              });
              if (!decision.confirmed) return;
              const payload = { user_id: userId, reason: decision.reason };
              const resp = await request({ method: "POST", url: buildV2("traffic-reset/reset-user"), data: payload, clearsDirty: true });
              setPreOutput("opsTrafficResult", resp);
              showToast("用户流量已重置", "success");
              await openOpsModule("traffic", { force: true });
            } catch (err) {
              setPreOutput("opsTrafficResult", err.message || "重置失败");
              showToast(err.message || "重置失败", "error");
            }
          });
        }

        if (historyBtn) {
          historyBtn.addEventListener("click", async () => {
            try {
              const userId = readInt("opsTrafficUserId", 0);
              if (!userId) throw new Error("请输入用户编号");
              const history = await request({ method: "GET", url: `${buildV2(`traffic-reset/user/${userId}/history`)}?limit=20` });
              setPreOutput("opsTrafficHistory", history);
              showToast("历史记录已加载", "success");
            } catch (err) {
              setPreOutput("opsTrafficHistory", err.message || "读取失败");
              showToast(err.message || "读取失败", "error");
            }
          });
        }
      }

      function bindOpsModule(moduleKey) {
        const controller = MODULE_CONTROLLERS[moduleKey];
        if (controller && typeof controller.bind === "function") controller.bind();
      }

      async function openOpsModule(moduleKey, options = {}) {
        const key = OPS_MODULE_MAP[moduleKey] ? moduleKey : "security";
        const controller = MODULE_CONTROLLERS[key];
        const force = Boolean(options.force);
        const targetIsActive = state.activeTab === "ops" && state.activeOpsModule === key;

        if (!state.token) {
          if (targetIsActive) {
            dom.opsWorkspace.innerHTML = '<p class="empty">请先登录管理员账号后加载模块。</p>';
          }
          return false;
        }

        if (!targetIsActive) {
          if (!force) {
            state.loadedOpsModules.delete(key);
            return false;
          }
          try {
            await controller.load(true);
            state.loadedOpsModules.add(key);
          } catch (err) {
            state.loadedOpsModules.delete(key);
            showToast(`操作已完成，但「${OPS_MODULE_MAP[key].name}」后台刷新失败`, "error");
          }
          return false;
        }

        if (state.opsDirty && force && !options.discardChanges) {
          const decision = await requestActionConfirmation({
            title: "放弃未保存修改",
            message: "刷新当前模块会丢失尚未保存的修改。",
            confirmLabel: "放弃并刷新"
          });
          if (!decision.confirmed) return false;
          resetOpsDirtyState();
        }

        const epoch = ++state.navigationEpoch;
        const currentMenuQuery = readValue("primaryOpsMenuSearch", "");
        if (currentMenuQuery && !moduleMatchesSearch(OPS_MODULE_MAP[key], currentMenuQuery) && dom.primaryOpsMenuSearch) {
          dom.primaryOpsMenuSearch.value = "";
        }
        renderPrimaryOpsMenu(readValue("primaryOpsMenuSearch", ""));
        setOpsModuleHeader(key);
        setActiveMenuButton("ops", key);

        dom.opsWorkspace.innerHTML = `<p class="empty">正在加载「${escapeHtml(OPS_MODULE_MAP[key].name)}」...</p>`;
        try {
          await controller.load(force);
          if (epoch !== state.navigationEpoch || state.activeTab !== "ops" || state.activeOpsModule !== key) {
            return false;
          }
          state.loadedOpsModules.add(key);
          dom.opsWorkspace.innerHTML = renderOpsModule(key);
          enhanceWorkspaceTables(dom.opsWorkspace, OPS_MODULE_MAP[key].name);
          resetOpsDirtyState();
          bindOpsModule(key);
          return true;
        } catch (err) {
          if (epoch !== state.navigationEpoch || state.activeTab !== "ops" || state.activeOpsModule !== key) {
            return false;
          }
          dom.opsWorkspace.innerHTML = `
            <section class="ops-card">
              <h4 class="ops-card-title">模块加载失败</h4>
              <p class="empty">${escapeHtml(err.message || "未知错误")}</p>
              <div class="actions">
                <button class="btn ghost" id="opsRetryModuleBtn">重试加载</button>
              </div>
            </section>
          `;
          const retryBtn = document.getElementById("opsRetryModuleBtn");
          if (retryBtn) {
            retryBtn.addEventListener("click", () => openOpsModule(key, { force: true }));
          }
          showToast(err.message || "模块加载失败", "error");
          return false;
        }
      }

      function bindTabs() {
        dom.menuTabs.addEventListener("click", (e) => {
          const btn = e.target.closest(".menu-btn");
          if (!btn) return;
          const tab = btn.dataset.tab;
          const moduleKey = btn.dataset.opsModule || "";
          activateView(tab || "overview", moduleKey, { syncRoute: true }).catch((err) => {
            showToast(err.message || "页面加载失败", "error");
          });
        });
      }

      async function loginWithPassword() {
        const email = dom.loginEmailInput.value.trim();
        const password = dom.loginPasswordInput.value;
        if (!email || !password) {
          setLoginError("请输入邮箱和密码");
          return;
        }

        try {
          dom.loginSubmitBtn.disabled = true;
          const cfg = await loadLoginCommConfig().catch(() => ({}));
          const payload = { email, password };

          if (isCaptchaEnabled(cfg)) {
            const provider = state.loginCaptchaProvider || getCaptchaProvider(cfg);
            if (provider === "turnstile") {
              if (!state.loginCaptchaToken) {
                throw new Error("请先完成 Turnstile 验证");
              }
              payload.turnstile_token = state.loginCaptchaToken;
            } else if (provider === "recaptcha") {
              if (!state.loginCaptchaToken) {
                throw new Error("请先完成 reCAPTCHA 验证");
              }
              payload.recaptcha_data = state.loginCaptchaToken;
            } else if (provider === "recaptcha-v3") {
              const siteKey = String(cfg.recaptcha_v3_site_key || "").trim();
              if (!siteKey) {
                throw new Error("reCAPTCHA v3 站点密钥未配置");
              }
              if (!state.loginCaptchaV3Ready) {
                await ensureRecaptchaLoaded("recaptcha-v3", siteKey);
                state.loginCaptchaV3Ready = true;
              }
              const token = await new Promise((resolve, reject) => {
                if (!window.grecaptcha || typeof window.grecaptcha.ready !== "function" || typeof window.grecaptcha.execute !== "function") {
                  reject(new Error("reCAPTCHA v3 未就绪"));
                  return;
                }
                window.grecaptcha.ready(() => {
                  window.grecaptcha.execute(siteKey, { action: "login" })
                    .then((t) => resolve(String(t || "").trim()))
                    .catch(() => reject(new Error("reCAPTCHA v3 执行失败")));
                });
              });
              if (!token) {
                throw new Error("reCAPTCHA v3 token 为空");
              }
              payload.recaptcha_v3_token = token;
            }
          }

          if (isPowEnabled(cfg)) {
            const proof = await ensureLoginPowProof();
            if (!proof) {
              throw new Error("防刷验证未就绪，请稍后重试");
            }
            payload.pow_id = proof.id;
            payload.pow_nonce = proof.nonce;
            payload.pow_hash = proof.hash;
            payload.pow_token = proof.token;
          }

          const body = await request({
            method: "POST",
            url: buildV1("passport/auth/login"),
            data: payload
          });
          const data = toData(body);
          const authData = data && data.auth_data;
          if (!authData) {
            throw new Error("登录返回信息不完整，请检查验证码和防刷验证设置");
          }
          updateToken(authData);
          persistSharedLegacyToken((data && data.token) || "");
          await syncAdminSession();
          showConsoleView();
          showToast("登录成功", "success");
          await loadAll().catch((err) => {
            showToast(err.message || "当前视图加载失败，请重试", "error");
          });
        } catch (err) {
          setLoginError(err.message || "登录失败");
          // Captcha/PoW token is usually single-use; on failure always reset and prepare again.
          resetLoginCaptchaWidget();
          state.loginPowProof = null;
          prepareLoginSecurity().catch(() => {});
        } finally {
          updateLoginSubmitAvailability();
        }
      }

      async function loadOverview(manual = false) {
        const requestEpoch = state.navigationEpoch;
        clearOverviewRefresh();
        updateOverviewCountdown();

        if (dom.refreshOverviewBtn) {
          dom.refreshOverviewBtn.disabled = true;
          dom.refreshOverviewBtn.setAttribute("aria-label", manual ? "正在刷新工作台" : "正在同步工作台");
        }
        setOverviewStatus(manual ? "手动拉取最新统计" : "正在同步后台总览", "neutral");
        if (dom.overviewHint) {
          dom.overviewHint.textContent = "正在同步...";
        }

        try {
          const body = await request({ method: "GET", url: buildV2("stat/getOverride") });
          const stats = (body && body.data) ? body.data : toData(body);
          if (requestEpoch !== state.navigationEpoch || state.activeTab !== "overview") return false;
          document.getElementById("kpiMonthIncome").textContent = money(stats.month_income);
          document.getElementById("kpiMonthUsers").textContent = formatCompactNumber(stats.month_register_total);
          document.getElementById("kpiPendingTicket").textContent = formatCompactNumber(stats.ticket_pending_total);
          document.getElementById("kpiOnlineUsers").textContent = formatCompactNumber(stats.online_users);

          if (dom.commandCenterDashboard) {
            dom.commandCenterDashboard.innerHTML = buildOverviewLanding({
              adminEmail: state.currentAdmin || dom.currentAdminText?.textContent || "-",
              securePath,
              stats,
            });
          }
          if (dom.commandCenterUpdatedAt) {
            dom.commandCenterUpdatedAt.textContent = formatAnyTimestamp(Math.floor(Date.now() / 1000));
          }
          state.overviewLoaded = true;
          setOverviewStatus("工作台已同步", "ok");
          if (dom.overviewHint) dom.overviewHint.textContent = "数据已更新";
          if (state.activeTab === "overview") scheduleOverviewRefresh(30);
          return true;
        } catch (err) {
          if (requestEpoch !== state.navigationEpoch || state.activeTab !== "overview") return false;
          setOverviewStatus("同步失败", "bad");
          if (dom.overviewHint) dom.overviewHint.textContent = err.message || "同步失败";
          throw err;
        } finally {
          if (dom.refreshOverviewBtn) {
            dom.refreshOverviewBtn.disabled = state.activeTab === "api";
            dom.refreshOverviewBtn.setAttribute("aria-label", "刷新当前视图");
          }
        }
      }

      function renderUsers() {
        if (!state.users.length) {
          dom.usersTableBody.innerHTML = '<tr><td colspan="8" class="empty">当前页无用户数据。</td></tr>';
          return;
        }

        dom.usersTableBody.innerHTML = state.users.map((u) => {
          const banned = Number(u.banned) === 1;
          const admin = Number(u.is_admin) === 1;
          const superAdmin = Number(u.is_super_admin) === 1;
          return `
            <tr>
              <td>${u.id}</td>
              <td>${escapeHtml(u.email)}</td>
              <td>${escapeHtml((u.plan && u.plan.name) || "-")}</td>
              <td>${money(u.balance)}</td>
              <td><span class="badge ${banned ? "warn" : "ok"}">${banned ? "已封禁" : "正常"}</span></td>
              <td>${escapeHtml((u.ban_reason || "").trim() || "-")}</td>
              <td>${superAdmin ? "超级管理员" : (admin ? "管理员" : "用户")}</td>
              <td>
                <div class="actions">
                  <button class="btn ${banned ? "ghost" : "warn"}" data-action="ban" data-id="${u.id}" data-email="${escapeHtml(u.email)}" data-banned="${banned ? 1 : 0}">${banned ? "解封" : "封禁"}</button>
                  <button class="btn ghost" data-action="secret" data-id="${u.id}" data-email="${escapeHtml(u.email)}">重置密钥</button>
                </div>
              </td>
            </tr>
          `;
        }).join("");
      }

      function buildUserFilters() {
        const filters = [];
        const search = String(state.usersSearch || "").trim();
        if (search) {
          if (state.usersSearchField === "id") {
            const id = Number(search);
            if (!Number.isInteger(id) || id <= 0) throw new Error("用户编号必须为正整数");
            filters.push({ id: "id", value: id });
          } else {
            filters.push({ id: "email", value: search });
          }
        }
        if (state.usersStatus === "active") filters.push({ id: "banned", value: false });
        if (state.usersStatus === "banned") filters.push({ id: "banned", value: true });
        return filters;
      }

      async function loadUsers() {
        const requestEpoch = ++state.usersRequestEpoch;
        state.usersLoading = true;
        dom.loadUsersBtn.disabled = true;
        dom.prevUsersBtn.disabled = true;
        dom.nextUsersBtn.disabled = true;
        dom.usersTableBody.innerHTML = '<tr><td colspan="8" class="empty">正在加载用户...</td></tr>';
        try {
          const body = await request({
            method: "POST",
            url: buildV2("user/fetch"),
            readOnly: true,
            data: {
              current: state.usersPage,
              pageSize: state.usersPageSize,
              filter: buildUserFilters(),
              sort: [{ id: "id", desc: true }]
            }
          });

          if (requestEpoch !== state.usersRequestEpoch) return false;

          state.users = Array.isArray(body.data) ? body.data : [];
          state.usersPage = Math.max(1, Number(body.current_page || state.usersPage || 1));
          state.usersPageSize = Math.max(1, Number(body.per_page || state.usersPageSize || 20));
          state.usersLastPage = Math.max(1, Number(body.last_page || 1));
          state.usersTotal = Math.max(0, Number(body.total || 0));
          state.usersLoaded = true;
          dom.usersPageSize.value = String(state.usersPageSize);
          dom.usersPageInfo.textContent = `${state.usersPage} / ${state.usersLastPage}`;
          dom.usersTotalText.textContent = `共 ${state.usersTotal.toLocaleString("zh-CN")} 位用户`;
          dom.prevUsersBtn.disabled = state.usersPage <= 1;
          dom.nextUsersBtn.disabled = state.usersPage >= state.usersLastPage;
          renderUsers();
        } catch (err) {
          if (requestEpoch !== state.usersRequestEpoch) return false;
          state.usersLoaded = false;
          dom.usersTableBody.innerHTML = `<tr><td colspan="8" class="empty">${escapeHtml(err.message || "用户加载失败")}</td></tr>`;
          throw err;
        } finally {
          if (requestEpoch === state.usersRequestEpoch) {
            state.usersLoading = false;
            dom.loadUsersBtn.disabled = false;
          }
        }
      }

      async function handleUserAction(e) {
        const btn = e.target.closest("button[data-action]");
        if (!btn) return;
        const action = btn.dataset.action;
        const id = Number(btn.dataset.id);
        const email = String(btn.dataset.email || `#${id}`);

        try {
          btn.disabled = true;
          if (action === "ban") {
            const banned = Number(btn.dataset.banned) === 1;
            const payload = { id, banned: banned ? 0 : 1 };
            const decision = await requestActionConfirmation({
              title: banned ? "解除用户封禁" : "封禁用户",
              message: banned
                ? `将恢复 ${email} 的账号访问。`
                : `将立即阻止 ${email} 登录和使用服务。`,
              confirmLabel: banned ? "确认解封" : "确认封禁",
              showReason: true,
              requireReason: !banned,
              reasonPlaceholder: banned ? "解封说明（可选）" : "填写封禁原因"
            });
            if (!decision.confirmed) return;
            if (decision.reason) payload.ban_reason = decision.reason;

            await request({ method: "POST", url: buildV2("user/update"), data: payload });
            showToast(banned ? "用户已解封" : "用户已封禁", "success");
          } else if (action === "secret") {
            const decision = await requestActionConfirmation({
              title: "重置订阅密钥",
              message: `${email} 的旧订阅地址将立即失效。`,
              confirmLabel: "确认重置"
            });
            if (!decision.confirmed) return;
            await request({ method: "POST", url: buildV2("user/resetSecret"), data: { id } });
            showToast("已重置该用户订阅密钥", "success");
          }
          await loadUsers();
        } catch (err) {
          showToast(err.message || "操作失败", "error");
        } finally {
          btn.disabled = false;
        }
      }

      async function sendApiRequest() {
        try {
          const version = dom.apiVersionSelect.value;
          const method = dom.apiMethodSelect.value;
          const endpoint = dom.apiEndpointInput.value.trim();
          if (!endpoint) {
            showToast("请输入请求地址", "error");
            return;
          }

          let url = endpoint;
          if (version === "v2") {
            url = buildV2(endpoint);
          } else if (version === "v1") {
            url = buildV1(endpoint);
          } else {
            const customUrl = new URL(endpoint, window.location.origin);
            if (customUrl.origin !== window.location.origin) {
              throw new Error("自定义地址仅允许当前站点，避免管理员令牌发送到外部域名");
            }
            url = `${customUrl.pathname}${customUrl.search}${customUrl.hash}`;
          }

          let bodyData;
          if (dom.apiBodyInput.value.trim()) {
            try {
              bodyData = JSON.parse(dom.apiBodyInput.value);
            } catch (err) {
              throw new Error("请求内容格式不正确，请按示例填写");
            }
          }

          if (method === "DELETE") {
            const decision = await requestActionConfirmation({
              title: "发送 DELETE 请求",
              message: `${url} 可能永久删除数据。`,
              confirmLabel: "确认发送"
            });
            if (!decision.confirmed) return;
          }

          const resp = await request({ method, url, data: bodyData });
          dom.apiResponseOutput.textContent = JSON.stringify(resp, null, 2);
          showToast("请求完成", "success");
        } catch (err) {
          dom.apiResponseOutput.textContent = String(err.message || err);
          showToast(err.message || "请求失败", "error");
        }
      }

      async function loadAll() {
        await loadActiveView({ force: false });
      }

      function bindEvents() {
        bindTabs();

        const restoreViewFromLocation = () => {
          restoreViewFromLocation.requested = true;
          if (restoreViewFromLocation.running) return;
          restoreViewFromLocation.running = true;
          (async () => {
            while (restoreViewFromLocation.requested) {
              restoreViewFromLocation.requested = false;
              const route = parseViewRoute();
              if (route.tab === state.activeTab && (route.tab !== "ops" || route.moduleKey === state.activeOpsModule)) {
                continue;
              }
              const changed = await activateView(route.tab, route.moduleKey, { syncRoute: false });
              if (changed === false) {
                syncViewRoute(state.activeTab, state.activeOpsModule, false);
              }
            }
          })()
            .catch((err) => {
              showToast(err.message || "页面加载失败", "error");
            })
            .finally(() => {
              restoreViewFromLocation.running = false;
            });
        };
        window.addEventListener("hashchange", restoreViewFromLocation);
        window.addEventListener("popstate", restoreViewFromLocation);

        dom.mobileNavToggle?.addEventListener("click", () => {
          setMobileNavigation(true);
          window.setTimeout(() => dom.primaryOpsMenuSearch?.focus(), 0);
        });
        dom.sidebarCloseBtn?.addEventListener("click", () => {
          setMobileNavigation(false);
          dom.mobileNavToggle?.focus();
        });
        dom.mobileNavBackdrop?.addEventListener("click", () => {
          setMobileNavigation(false);
          dom.mobileNavToggle?.focus();
        });
        window.matchMedia("(max-width: 900px)").addEventListener("change", () => {
          setMobileNavigation(false);
        });
        dom.opsWorkspace?.addEventListener("input", (event) => {
          if (event.target.matches("input, select, textarea")) markOpsDirty(event.target);
        });
        window.addEventListener("beforeunload", (event) => {
          if (!state.opsDirty) return;
          event.preventDefault();
          event.returnValue = "";
        });

        dom.actionDialogForm?.addEventListener("submit", (event) => {
          event.preventDefault();
          const reason = String(dom.actionDialogReason.value || "").trim();
          if (dom.actionDialogReason.dataset.required === "true" && !reason) {
            dom.actionDialogError.textContent = "请填写原因";
            dom.actionDialogReason.setAttribute("aria-invalid", "true");
            dom.actionDialogReason.focus();
            return;
          }
          settleActionDialog({ confirmed: true, reason });
        });
        dom.actionDialogCancel?.addEventListener("click", () => {
          settleActionDialog({ confirmed: false, reason: "" });
        });
        dom.actionDialogClose?.addEventListener("click", () => {
          settleActionDialog({ confirmed: false, reason: "" });
        });
        dom.actionDialog?.addEventListener("cancel", (event) => {
          event.preventDefault();
          settleActionDialog({ confirmed: false, reason: "" });
        });
        dom.actionDialogReason?.addEventListener("input", () => {
          dom.actionDialogError.textContent = "";
          dom.actionDialogReason.setAttribute("aria-invalid", "false");
        });

        document.addEventListener("keydown", (event) => {
          if (event.key === "Escape") {
            if (document.body.classList.contains("nav-open")) {
              event.preventDefault();
              setMobileNavigation(false);
              dom.mobileNavToggle?.focus();
            }
            return;
          }
          if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
            event.preventDefault();
            setMobileNavigation(true);
            dom.primaryOpsMenuSearch?.focus();
          }
        });

        if (dom.themeDarkBtn) {
          dom.themeDarkBtn.addEventListener("click", () => applyAdminTheme("dark"));
        }

        if (dom.themeLightBtn) {
          dom.themeLightBtn.addEventListener("click", () => applyAdminTheme("light"));
        }

        dom.sidebarThemeBtn?.addEventListener("click", () => {
          applyAdminTheme(state.adminTheme === "dark" ? "light" : "dark");
        });

        if (dom.primaryOpsMenuSearch) {
          dom.primaryOpsMenuSearch.addEventListener("input", (e) => {
            renderPrimaryOpsMenu(e.target.value || "");
            setActiveMenuButton(state.activeTab, state.activeTab === "ops" ? state.activeOpsModule : "");
          });
        }

        dom.loginSubmitBtn.addEventListener("click", loginWithPassword);
        dom.loginPasswordInput.addEventListener("keydown", (e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            loginWithPassword();
          }
        });
        dom.loginEmailInput.addEventListener("keydown", (e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            loginWithPassword();
          }
        });

        dom.logoutBtn.addEventListener("click", async () => {
          if (state.opsDirty) {
            const decision = await requestActionConfirmation({
              title: "退出超级管理员",
              message: "当前模块有未保存的修改，退出后这些修改会丢失。",
              confirmLabel: "放弃并退出"
            });
            if (!decision.confirmed) return;
          }
          showLoginView("已退出登录", { clearSharedAuth: true });
          showToast("已退出登录", "info");
        });

        dom.refreshOverviewBtn.addEventListener("click", async () => {
          try {
            const refreshed = await loadActiveView({ force: true });
            if (refreshed !== false) showToast("当前视图已刷新", "success");
          } catch (err) {
            showToast(err.message || "刷新失败", "error");
          }
        });

        if (dom.openCommandCenterBtn) {
          dom.openCommandCenterBtn.addEventListener("click", () => {
            showToast("已独立打开超级管理员监控大屏", "info");
          });
        }

        if (dom.commandCenterDashboard) {
          dom.commandCenterDashboard.addEventListener("click", (e) => {
            const jumpBtn = e.target.closest("[data-overview-jump]");
            if (jumpBtn) {
              const tab = jumpBtn.dataset.overviewJump;
              const moduleKey = jumpBtn.dataset.overviewModule || "";
              activateView(tab, moduleKey, { syncRoute: true }).catch((err) => {
                showToast(err.message || "页面加载失败", "error");
              });
              return;
            }

            const openBtn = e.target.closest("[data-overview-open]");
            if (openBtn && openBtn.dataset.overviewOpen === "command-center") {
              if (dom.openCommandCenterBtn) {
                window.open(dom.openCommandCenterBtn.href, "_blank", "noopener,noreferrer");
                showToast("已独立打开超级管理员监控大屏", "info");
              }
            }
          });
        }

        dom.loadUsersBtn.addEventListener("click", async () => {
          try {
            await loadUsers();
            showToast("用户列表已更新", "success");
          } catch (err) {
            showToast(err.message || "加载失败", "error");
          }
        });

        const applyUserFilters = async () => {
          state.usersSearchField = dom.usersSearchField.value === "id" ? "id" : "email";
          state.usersSearch = dom.usersSearchInput.value.trim();
          state.usersStatus = ["active", "banned"].includes(dom.usersStatusFilter.value)
            ? dom.usersStatusFilter.value
            : "all";
          state.usersPageSize = Math.max(1, Number(dom.usersPageSize.value || 20));
          state.usersPage = 1;
          await loadUsers();
        };

        dom.usersSearchBtn?.addEventListener("click", () => {
          applyUserFilters().catch((err) => showToast(err.message || "查询失败", "error"));
        });
        dom.usersSearchInput?.addEventListener("keydown", (event) => {
          if (event.key !== "Enter") return;
          event.preventDefault();
          applyUserFilters().catch((err) => showToast(err.message || "查询失败", "error"));
        });
        dom.usersClearBtn?.addEventListener("click", () => {
          dom.usersSearchField.value = "email";
          dom.usersSearchInput.value = "";
          dom.usersStatusFilter.value = "all";
          dom.usersPageSize.value = "20";
          applyUserFilters().catch((err) => showToast(err.message || "加载失败", "error"));
        });
        dom.usersStatusFilter?.addEventListener("change", () => {
          applyUserFilters().catch((err) => showToast(err.message || "筛选失败", "error"));
        });
        dom.usersPageSize?.addEventListener("change", () => {
          applyUserFilters().catch((err) => showToast(err.message || "加载失败", "error"));
        });

        dom.prevUsersBtn.addEventListener("click", async () => {
          if (state.usersPage <= 1) return;
          state.usersPage -= 1;
          try {
            await loadUsers();
          } catch (err) {
            showToast(err.message || "加载失败", "error");
          }
        });

        dom.nextUsersBtn.addEventListener("click", async () => {
          if (state.usersPage >= state.usersLastPage) return;
          state.usersPage += 1;
          try {
            await loadUsers();
          } catch (err) {
            showToast(err.message || "加载失败", "error");
          }
        });

        dom.usersTableBody.addEventListener("click", handleUserAction);

        dom.sendApiBtn.addEventListener("click", sendApiRequest);

        if (dom.opsRefreshBtn) {
          dom.opsRefreshBtn.addEventListener("click", () => {
            openOpsModule(state.activeOpsModule || "security", { force: true }).catch((err) => {
              showToast(err.message || "刷新失败", "error");
            });
          });
        }

        if (dom.opsOpenApiBtn) {
          dom.opsOpenApiBtn.addEventListener("click", () => {
            activateView("api", "", { syncRoute: true }).catch(() => {});
          });
        }

        dom.presetThemeBtn.addEventListener("click", () => {
          dom.apiVersionSelect.value = "v2";
          dom.apiMethodSelect.value = "GET";
          dom.apiEndpointInput.value = "system/getSystemStatus";
          dom.apiBodyInput.value = "";
          showToast("已填充系统状态请求", "info");
        });

        dom.presetLimitBtn.addEventListener("click", () => {
          dom.apiVersionSelect.value = "v1";
          dom.apiMethodSelect.value = "PUT";
          dom.apiEndpointInput.value = "admin/users/1/concurrent-ip-limit";
          dom.apiBodyInput.value = JSON.stringify({ concurrent_ip_limit: 2 }, null, 2);
          showToast("已填充并发 IP 限制请求", "info");
        });
      }

      async function init() {
        const initialRoute = parseViewRoute();
        state.activeTab = initialRoute.tab;
        if (initialRoute.moduleKey) state.activeOpsModule = initialRoute.moduleKey;
        applyAdminTheme(localStorage.getItem(ADMIN_THEME_STORAGE_KEY) || "light", false);
        renderPrimaryOpsMenu();
        bindEvents();
        setOpsModuleHeader(state.activeOpsModule || "security");
        await activateView(state.activeTab, state.activeOpsModule, {
          syncRoute: true,
          replaceRoute: true,
          load: false
        });

        const storedToken = readSharedAuthToken();
        if (storedToken) {
          updateToken(storedToken, false);
          try {
            await syncAdminSession();
            showConsoleView();
          } catch (err) {
            showLoginView(
              err.message || "登录状态已失效，请重新登录。",
              { clearSharedAuth: Number(err?.status || 0) === 401 }
            );
            return;
          }
          await loadAll().catch((err) => {
            showToast(err.message || "当前视图加载失败，请重试", "error");
          });
          return;
        }

        showLoginView();
      }

      window.addEventListener("storage", (event) => {
        if (event.key === ADMIN_THEME_STORAGE_KEY) {
          applyAdminTheme(event.newValue || "light", false);
          return;
        }

        if (![...sharedAuthKeys, SHARED_LEGACY_TOKEN_KEY, SHARED_PROFILE_KEY].includes(event.key || "")) {
          return;
        }

        const nextToken = readSharedAuthToken();
        const currentToken = normalizeBearer(state.token);

        if (!nextToken) {
          if (currentToken) {
            showLoginView("登录状态已同步退出。");
          }
          return;
        }

        if (nextToken === currentToken) {
          return;
        }

        clearToken(false);
        updateToken(nextToken, false);
        syncAdminSession()
          .then(async (me) => {
            if (!me) return;
            showConsoleView();
            await loadAll().catch((err) => {
              showToast(err.message || "当前视图加载失败，请重试", "error");
            });
          })
          .catch((err) => {
            showLoginView(
              err.message || "登录状态同步失败，请重新登录。",
              { clearSharedAuth: Number(err?.status || 0) === 401 }
            );
          });
      });

      init();
    })();
