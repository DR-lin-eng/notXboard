(() => {
  const CONFIG_URL = "/api/v1/guest/comm/config";
  const POW_CHALLENGE_URL = "/api/v1/passport/auth/pow-challenge";
  const LOGIN_PATH = "/passport/auth/login";
  const REGISTER_PATH = "/passport/auth/register";

  const TURNSTILE_CONTAINER_ID = "portal-turnstile-widget";
  const TURNSTILE_TOKEN_KEY = "__portalTurnstileToken";
  const TURNSTILE_CONFIG_KEY = "__portalTurnstileConfig";
  const TURNSTILE_WIDGET_KEY = "__portalTurnstileWidgetId";
  const TURNSTILE_PATCH_KEY = "__portalTurnstilePatched";
  const TURNSTILE_SCRIPT_ID = "portal-turnstile-script";

  const POW_PROOF_KEY = "__portalPowProof";

  let configPromise = null;
  let observer = null;
  let powPromise = null;

  const getAuthRoute = () => {
    const hash = window.location.hash || "";
    if (hash.startsWith("#/register") || hash.startsWith("#/sign-up")) {
      return "register";
    }
    if (hash.startsWith("#/login") || hash.startsWith("#/sign-in")) {
      return "login";
    }
    return "";
  };

  const isLoginRoute = () => getAuthRoute() === "login";
  const isAuthRoute = () => !!getAuthRoute();

  const getInviteCodeFromHash = () => {
    const hash = window.location.hash || "";
    const queryText = hash.includes("?") ? hash.slice(hash.indexOf("?") + 1) : "";
    if (!queryText) {
      return "";
    }
    try {
      const params = new URLSearchParams(queryText);
      return String(params.get("invite_code") || "").trim();
    } catch (e) {
      return "";
    }
  };

  const loadConfig = () => {
    if (configPromise) {
      return configPromise;
    }
    if (window[TURNSTILE_CONFIG_KEY]) {
      return Promise.resolve(window[TURNSTILE_CONFIG_KEY]);
    }
    configPromise = fetch(CONFIG_URL, { credentials: "same-origin" })
      .then((resp) => (resp.ok ? resp.json() : null))
      .then((data) => {
        const config = data && data.data ? data.data : null;
        if (config) {
          window[TURNSTILE_CONFIG_KEY] = config;
        }
        return config;
      })
      .catch(() => null);
    return configPromise;
  };

  const isTurnstileEnabled = (config) => {
    return config && Number(config.is_captcha) === 1 && config.captcha_type === "turnstile";
  };

  const isPowEnabled = (config) => {
    return config && Number(config.pow_enable) === 1;
  };

  const getTurnstileToken = () => window[TURNSTILE_TOKEN_KEY] || "";
  const setTurnstileToken = (token) => {
    window[TURNSTILE_TOKEN_KEY] = token || "";
    updateLoginButtonState();
  };

  const consumeTurnstileToken = () => {
    const token = getTurnstileToken();
    if (!token) {
      return "";
    }
    setTurnstileToken("");
    const widgetId = window[TURNSTILE_WIDGET_KEY];
    if (window.turnstile && typeof widgetId === "string") {
      window.turnstile.reset(widgetId);
    }
    return token;
  };

  const ensureTurnstileScript = () => {
    if (window.turnstile) {
      return Promise.resolve();
    }
    return new Promise((resolve, reject) => {
      if (document.getElementById(TURNSTILE_SCRIPT_ID)) {
        const checkReady = () => {
          if (window.turnstile) {
            resolve();
          } else {
            setTimeout(checkReady, 50);
          }
        };
        checkReady();
        return;
      }
      const script = document.createElement("script");
      script.id = TURNSTILE_SCRIPT_ID;
      script.async = true;
      script.defer = true;
      script.src = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
      script.onload = () => resolve();
      script.onerror = () => reject(new Error("Turnstile script failed to load"));
      document.head.appendChild(script);
    });
  };

  const findLoginContainer = () => {
    return document.querySelector(".portal-auth-box") || document.querySelector("#root");
  };

  const ensureTurnstileContainer = () => {
    const root = findLoginContainer();
    if (!root) {
      return null;
    }
    let container = document.getElementById(TURNSTILE_CONTAINER_ID);
    if (container) {
      return container;
    }
    container = document.createElement("div");
    container.id = TURNSTILE_CONTAINER_ID;
    container.className = "portal-turnstile";

    const submitButton = root.querySelector("button[type='submit']") || root.querySelector("button");
    if (submitButton && submitButton.parentNode) {
      submitButton.parentNode.insertBefore(container, submitButton);
    } else {
      root.appendChild(container);
    }
    return container;
  };

  const renderTurnstile = async () => {
    if (!isAuthRoute()) {
      return;
    }
    const config = await loadConfig();
    if (!isTurnstileEnabled(config)) {
      return;
    }
    if (!config.turnstile_site_key) {
      return;
    }
    await ensureTurnstileScript();
    const existingContainer = document.getElementById(TURNSTILE_CONTAINER_ID);
    if (!existingContainer && window[TURNSTILE_WIDGET_KEY]) {
      window[TURNSTILE_WIDGET_KEY] = null;
    }
    const container = ensureTurnstileContainer();
    if (!container || !window.turnstile) {
      return;
    }
    if (window[TURNSTILE_WIDGET_KEY] && container.childNodes.length) {
      return;
    }
    window[TURNSTILE_WIDGET_KEY] = window.turnstile.render(container, {
      sitekey: config.turnstile_site_key,
      callback: (token) => setTurnstileToken(token),
      "expired-callback": () => setTurnstileToken(""),
      "error-callback": () => setTurnstileToken(""),
    });
    updateLoginButtonState();
  };

  const bufferToHex = (buffer) => {
    const bytes = new Uint8Array(buffer);
    let hex = "";
    for (let i = 0; i < bytes.length; i += 1) {
      hex += bytes[i].toString(16).padStart(2, "0");
    }
    return hex;
  };

  const sha256Fallback = (message) => {
    const utf8 = unescape(encodeURIComponent(message));
    const K = [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    const words = [];
    for (let i = 0; i < utf8.length; i += 1) {
      words[i >> 2] |= utf8.charCodeAt(i) << (24 - (i % 4) * 8);
    }
    const bitLength = utf8.length * 8;
    words[bitLength >> 5] |= 0x80 << (24 - (bitLength % 32));
    words[((bitLength + 64 >> 9) << 4) + 15] = bitLength;

    let a = 0x6a09e667;
    let b = 0xbb67ae85;
    let c = 0x3c6ef372;
    let d = 0xa54ff53a;
    let e = 0x510e527f;
    let f = 0x9b05688c;
    let g = 0x1f83d9ab;
    let h = 0x5be0cd19;

    for (let i = 0; i < words.length; i += 16) {
      const w = new Array(64);
      for (let j = 0; j < 16; j += 1) {
        w[j] = words[i + j] | 0;
      }
      for (let j = 16; j < 64; j += 1) {
        const s0 = (w[j - 15] >>> 7) | (w[j - 15] << 25);
        const s1 = (w[j - 15] >>> 18) | (w[j - 15] << 14);
        const s2 = w[j - 15] >>> 3;
        const s3 = (w[j - 2] >>> 17) | (w[j - 2] << 15);
        const s4 = (w[j - 2] >>> 19) | (w[j - 2] << 13);
        const s5 = w[j - 2] >>> 10;
        w[j] = (w[j - 16] + (s0 ^ s1 ^ s2) + w[j - 7] + (s3 ^ s4 ^ s5)) | 0;
      }

      let aa = a;
      let bb = b;
      let cc = c;
      let dd = d;
      let ee = e;
      let ff = f;
      let gg = g;
      let hh = h;

      for (let j = 0; j < 64; j += 1) {
        const s0 = (aa >>> 2) | (aa << 30);
        const s1 = (aa >>> 13) | (aa << 19);
        const s2 = (aa >>> 22) | (aa << 10);
        const maj = (aa & bb) ^ (aa & cc) ^ (bb & cc);
        const t2 = (s0 ^ s1 ^ s2) + maj;

        const s3 = (ee >>> 6) | (ee << 26);
        const s4 = (ee >>> 11) | (ee << 21);
        const s5 = (ee >>> 25) | (ee << 7);
        const ch = (ee & ff) ^ (~ee & gg);
        const t1 = (hh + (s3 ^ s4 ^ s5) + ch + K[j] + w[j]) | 0;

        hh = gg;
        gg = ff;
        ff = ee;
        ee = (dd + t1) | 0;
        dd = cc;
        cc = bb;
        bb = aa;
        aa = (t1 + t2) | 0;
      }

      a = (a + aa) | 0;
      b = (b + bb) | 0;
      c = (c + cc) | 0;
      d = (d + dd) | 0;
      e = (e + ee) | 0;
      f = (f + ff) | 0;
      g = (g + gg) | 0;
      h = (h + hh) | 0;
    }

    const hash = [a, b, c, d, e, f, g, h]
      .map((value) => (value >>> 0).toString(16).padStart(8, "0"))
      .join("");
    return hash;
  };

  const sha256Hex = async (message) => {
    if (window.crypto && window.crypto.subtle && window.TextEncoder) {
      const data = new TextEncoder().encode(message);
      const digest = await window.crypto.subtle.digest("SHA-256", data);
      return bufferToHex(digest);
    }
    return sha256Fallback(message);
  };

  const yieldControl = () => {
    return new Promise((resolve) => {
      if (window.requestIdleCallback) {
        window.requestIdleCallback(() => resolve());
      } else {
        setTimeout(resolve, 0);
      }
    });
  };

  const setPowProof = (proof) => {
    window[POW_PROOF_KEY] = proof || null;
    updateLoginButtonState();
  };

  const getPowProof = () => window[POW_PROOF_KEY] || null;

  const consumePowProof = () => {
    const proof = getPowProof();
    setPowProof(null);
    if (proof) {
      ensurePowProof();
    }
    return proof;
  };

  const loadPowChallenge = async () => {
    const response = await fetch(POW_CHALLENGE_URL, { credentials: "same-origin" });
    if (!response.ok) {
      return null;
    }
    const json = await response.json();
    return json && json.data ? json.data : null;
  };

  const computePowProof = async (challenge) => {
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
      if (nonce % 200 === 0) {
        await yieldControl();
      }
    }
  };

  const ensurePowProof = async () => {
    const config = await loadConfig();
    if (!isPowEnabled(config) || !isAuthRoute()) {
      setPowProof(null);
      return null;
    }
    const current = getPowProof();
    if (current && current.expires_at && current.expires_at > Math.floor(Date.now() / 1000) + 5) {
      return current;
    }
    if (powPromise) {
      return powPromise;
    }
    powPromise = (async () => {
      const challenge = await loadPowChallenge();
      if (!challenge) {
        setPowProof(null);
        return null;
      }
      const proof = await computePowProof(challenge);
      setPowProof(proof);
      powPromise = null;
      return proof;
    })();
    return powPromise;
  };

  const injectPayloadIntoBody = (body, payload) => {
    const keys = Object.keys(payload).filter((key) => payload[key]);
    if (keys.length === 0) {
      return { body, changed: false };
    }
    if (body == null) {
      return { body: JSON.stringify(payload), changed: true };
    }
    if (body instanceof FormData) {
      keys.forEach((key) => body.set(key, payload[key]));
      return { body, changed: true };
    }
    if (body instanceof URLSearchParams) {
      keys.forEach((key) => body.set(key, payload[key]));
      return { body, changed: true };
    }
    if (typeof body === "string") {
      try {
        const json = JSON.parse(body);
        if (json && typeof json === "object") {
          keys.forEach((key) => {
            json[key] = payload[key];
          });
          return { body: JSON.stringify(json), changed: true };
        }
      } catch (e) {
        const params = new URLSearchParams(body);
        keys.forEach((key) => params.set(key, payload[key]));
        return { body: params.toString(), changed: true };
      }
    }
    if (typeof body === "object") {
      keys.forEach((key) => {
        body[key] = payload[key];
      });
      return { body, changed: true };
    }
    return { body, changed: false };
  };

  const buildAuthPayload = async (url) => {
    const config = await loadConfig();
    const payload = {};

    if (isPowEnabled(config)) {
      const proof = await ensurePowProof();
      if (proof) {
        const consumed = consumePowProof();
        if (consumed) {
          payload.pow_id = consumed.id;
          payload.pow_nonce = consumed.nonce;
          payload.pow_hash = consumed.hash;
          payload.pow_token = consumed.token;
        }
      }
    }

    if (isTurnstileEnabled(config)) {
      const token = getTurnstileToken();
      if (token) {
        payload.turnstile_token = consumeTurnstileToken();
      }
    }

    if (typeof url === "string" && url.indexOf(REGISTER_PATH) !== -1) {
      const inviteCode = getInviteCodeFromHash();
      if (inviteCode) {
        payload.invite_code = inviteCode;
      }
    }

    return payload;
  };

  const shouldHandleUrl = (url) => {
    return typeof url === "string" && (url.indexOf(LOGIN_PATH) !== -1 || url.indexOf(REGISTER_PATH) !== -1);
  };

  const patchNetwork = () => {
    if (window[TURNSTILE_PATCH_KEY]) {
      return;
    }
    window[TURNSTILE_PATCH_KEY] = true;

    const originalFetch = window.fetch;
    if (typeof originalFetch === "function") {
      window.fetch = async (input, init) => {
        const url = typeof input === "string" ? input : input && input.url;
        if (!shouldHandleUrl(url)) {
          return originalFetch(input, init);
        }
        const payload = await buildAuthPayload(url);
        const nextInit = init ? { ...init } : {};
        const result = injectPayloadIntoBody(nextInit.body, payload);
        if (result.changed) {
          nextInit.body = result.body;
        }
        if (input instanceof Request) {
          return originalFetch(new Request(input, nextInit));
        }
        return originalFetch(input, nextInit);
      };
    }

    const originalOpen = XMLHttpRequest.prototype.open;
    const originalSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function (method, url) {
      this.__portalUrl = url;
      return originalOpen.apply(this, arguments);
    };
    XMLHttpRequest.prototype.send = function (body) {
      if (!shouldHandleUrl(this.__portalUrl || "")) {
        return originalSend.call(this, body);
      }
      const xhr = this;
      buildAuthPayload(this.__portalUrl || "")
        .then((payload) => {
          const result = injectPayloadIntoBody(body, payload);
          originalSend.call(xhr, result.body);
        })
        .catch(() => {
          originalSend.call(xhr, body);
        });
    };
  };

  const updateLoginButtonState = () => {
    if (!isAuthRoute()) {
      return;
    }
    const config = window[TURNSTILE_CONFIG_KEY];
    const root = findLoginContainer();
    if (!root) {
      return;
    }
    const submitButton = root.querySelector("button[type='submit']") || root.querySelector("button");
    if (!submitButton) {
      return;
    }

    const turnstileReady = !isTurnstileEnabled(config) || !!getTurnstileToken();
    const powReady = !isPowEnabled(config) || !!getPowProof();
    const canSubmit = turnstileReady && powReady;

    submitButton.disabled = !canSubmit;

    if (!turnstileReady) {
      submitButton.setAttribute("data-turnstile-locked", "true");
    } else {
      submitButton.removeAttribute("data-turnstile-locked");
    }

    if (!powReady) {
      submitButton.setAttribute("data-pow-locked", "true");
    } else {
      submitButton.removeAttribute("data-pow-locked");
    }
  };

  const ensureObserver = () => {
    if (observer) {
      return;
    }
    observer = new MutationObserver(() => {
      if (!isAuthRoute()) {
        return;
      }
      renderTurnstile();
      ensurePowProof();
      updateLoginButtonState();
    });
    observer.observe(document.body, { childList: true, subtree: true });
  };

  const boot = () => {
    patchNetwork();
    ensureObserver();
    renderTurnstile();
    ensurePowProof();
    updateLoginButtonState();
  };

  window.addEventListener("hashchange", () => {
    renderTurnstile();
    ensurePowProof();
    updateLoginButtonState();
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
