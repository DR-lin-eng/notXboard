const VIEW = document.getElementById('view');
const logoutBtn = document.getElementById('logoutBtn');
const NAV_LINKS = Array.from(document.querySelectorAll('.nav a[href^="#/"]'));
const AUTH_ONLY_LINKS = Array.from(document.querySelectorAll('.nav a[data-auth-only="1"]'));
const APP_CONTAINER = document.querySelector('.container');
const APP_ROOT = document.getElementById('app');
const APP_TOPBAR = document.querySelector('.topbar');
const APP_FOOTER = document.querySelector('.footer');
const NAV_TOGGLE = document.getElementById('navToggle');
const NAV_SCRIM = document.getElementById('navScrim');

const SHARED_AUTH_STORAGE_KEYS = Object.freeze([
  'auth_data',
  'PORTAL_ACCESS_TOKEN',
  'Portal_access_token',
  'access_token',
]);

function normalizeStoredBearer(token) {
  const clean = String(token || '').trim();
  if (!clean) return '';
  return clean.toLowerCase().startsWith('bearer ') ? clean : `Bearer ${clean}`;
}

function readSharedAuthData() {
  for (const key of SHARED_AUTH_STORAGE_KEYS) {
    const value = normalizeStoredBearer(localStorage.getItem(key) || '');
    if (value) return value;
  }
  return '';
}

function writeSharedAuthData(value) {
  const token = normalizeStoredBearer(value);
  SHARED_AUTH_STORAGE_KEYS.forEach((key) => {
    if (token) {
      localStorage.setItem(key, token);
    } else {
      localStorage.removeItem(key);
    }
  });
}

const store = {
  get auth() { return readSharedAuthData(); },
  set auth(v) { writeSharedAuthData(v); },
  get legacyToken() { return localStorage.getItem('token') || ''; },
  set legacyToken(v) { localStorage.setItem('token', v || ''); },
  get me() {
    try { return JSON.parse(localStorage.getItem('me') || 'null'); } catch { return null; }
  },
  set me(v) { localStorage.setItem('me', JSON.stringify(v || null)); }
};

const TURNSTILE_SCRIPT_ID = 'maintainable-turnstile-script';
const TURNSTILE_PRECONNECT_ID = 'maintainable-turnstile-preconnect';
const RECAPTCHA_SCRIPT_ID = 'maintainable-recaptcha-script';
const RECAPTCHA_PRECONNECT_ID = 'maintainable-recaptcha-preconnect';
const POW_CHALLENGE_URL = '/api/v1/passport/auth/pow-challenge';
const loginCaptchaState = {
  config: null,
  loginToken: '',
  registerToken: '',
  loginPowProof: null,
  registerPowProof: null,
  turnstileLoginWidgetId: null,
  turnstileRegisterWidgetId: null,
  recaptchaLoginWidgetId: null,
  recaptchaRegisterWidgetId: null,
  turnstileScriptPromise: null,
  recaptchaScriptPromise: null,
  recaptchaScriptSrc: '',
  recaptchaV3Ready: false,
  powPromises: { login: null, register: null },
};
let guestCommConfigCache = null;
let guestCommConfigPromise = null;
let activeViewCleanup = null;

function clearViewCleanup() {
  if (typeof activeViewCleanup === 'function') {
    try {
      activeViewCleanup();
    } catch (_) {
      // Ignore view teardown failures and continue rendering next route.
    }
  }
  activeViewCleanup = null;
}

function registerViewCleanup(cleanup) {
  activeViewCleanup = typeof cleanup === 'function' ? cleanup : null;
}

const NODE_PROTOCOL_LABELS = Object.freeze({
  vmess: 'VMess',
  vless: 'VLESS',
  trojan: 'Trojan',
  shadowsocks: 'Shadowsocks',
  hysteria: 'Hysteria',
  hysteria2: 'Hysteria2',
  tuic: 'TUIC',
  anytls: 'AnyTLS',
  socks: 'SOCKS',
  http: 'HTTP',
  naive: 'Naive',
  mieru: 'Mieru',
});

const DEFAULT_NODE_PROTOCOL_OPTIONS = Object.keys(NODE_PROTOCOL_LABELS).map((key) => ({
  value: key,
  label: NODE_PROTOCOL_LABELS[key],
}));

const V2BX_DEPLOYABLE_PROTOCOLS = Object.freeze(new Set([
  'vmess',
  'vless',
  'trojan',
  'shadowsocks',
  'hysteria',
  'hysteria2',
  'tuic',
  'anytls',
]));

const PLAN_PRICE_FIELDS = Object.freeze([
  { key: 'month_price', label: '月付' },
  { key: 'quarter_price', label: '季付' },
  { key: 'half_year_price', label: '半年付' },
  { key: 'year_price', label: '年付' },
  { key: 'two_year_price', label: '两年付' },
  { key: 'three_year_price', label: '三年付' },
  { key: 'onetime_price', label: '一次性' },
  { key: 'reset_price', label: '重置流量' },
]);

const TRUST_LEVEL_OPTIONS = Object.freeze([0, 1, 2, 3, 4]);
const NODE_LOCATION_OPTIONS = Object.freeze([
  { code: 'HK', name: 'Hong Kong 香港' },
  { code: 'TW', name: 'Taiwan 台湾' },
  { code: 'JP', name: 'Japan 日本' },
  { code: 'KR', name: 'South Korea 韩国' },
  { code: 'SG', name: 'Singapore 新加坡' },
  { code: 'MY', name: 'Malaysia 马来西亚' },
  { code: 'TH', name: 'Thailand 泰国' },
  { code: 'VN', name: 'Vietnam 越南' },
  { code: 'ID', name: 'Indonesia 印度尼西亚' },
  { code: 'PH', name: 'Philippines 菲律宾' },
  { code: 'IN', name: 'India 印度' },
  { code: 'AE', name: 'United Arab Emirates 阿联酋' },
  { code: 'TR', name: 'Turkey 土耳其' },
  { code: 'IL', name: 'Israel 以色列' },
  { code: 'US', name: 'United States 美国' },
  { code: 'CA', name: 'Canada 加拿大' },
  { code: 'MX', name: 'Mexico 墨西哥' },
  { code: 'BR', name: 'Brazil 巴西' },
  { code: 'AR', name: 'Argentina 阿根廷' },
  { code: 'CL', name: 'Chile 智利' },
  { code: 'GB', name: 'United Kingdom 英国' },
  { code: 'DE', name: 'Germany 德国' },
  { code: 'FR', name: 'France 法国' },
  { code: 'NL', name: 'Netherlands 荷兰' },
  { code: 'BE', name: 'Belgium 比利时' },
  { code: 'LU', name: 'Luxembourg 卢森堡' },
  { code: 'ES', name: 'Spain 西班牙' },
  { code: 'IT', name: 'Italy 意大利' },
  { code: 'CH', name: 'Switzerland 瑞士' },
  { code: 'AT', name: 'Austria 奥地利' },
  { code: 'SE', name: 'Sweden 瑞典' },
  { code: 'NO', name: 'Norway 挪威' },
  { code: 'FI', name: 'Finland 芬兰' },
  { code: 'PL', name: 'Poland 波兰' },
  { code: 'CZ', name: 'Czechia 捷克' },
  { code: 'RO', name: 'Romania 罗马尼亚' },
  { code: 'BG', name: 'Bulgaria 保加利亚' },
  { code: 'UA', name: 'Ukraine 乌克兰' },
  { code: 'RU', name: 'Russia 俄罗斯' },
  { code: 'ZA', name: 'South Africa 南非' },
  { code: 'AU', name: 'Australia 澳大利亚' },
  { code: 'NZ', name: 'New Zealand 新西兰' },
]);

// TCPing Agent probe location: requires explicit ownership region, CN mainland must include province.
const TCPING_AGENT_LOCATION_OPTIONS = Object.freeze([
  { code: 'CN', name: 'China Mainland 中国大陆' },
  ...NODE_LOCATION_OPTIONS,
  { code: 'MO', name: 'Macau 澳门' },
]);

const CN_PROVINCE_OPTIONS = Object.freeze([
  '北京',
  '天津',
  '上海',
  '重庆',
  '河北',
  '山西',
  '辽宁',
  '吉林',
  '黑龙江',
  '江苏',
  '浙江',
  '安徽',
  '福建',
  '江西',
  '山东',
  '河南',
  '湖北',
  '湖南',
  '广东',
  '海南',
  '四川',
  '贵州',
  '云南',
  '陕西',
  '甘肃',
  '青海',
  '内蒙古',
  '广西',
  '西藏',
  '宁夏',
  '新疆',
]);
const TCPING_MONITOR_RANGE_OPTIONS = Object.freeze([
  { hours: 24, label: '24 小时' },
  { hours: 72, label: '72 小时' },
  { hours: 168, label: '7 天' },
]);

const SUBSCRIBE_TEMPLATE_EDITORS = Object.freeze([
  { key: 'singbox', configKey: 'subscribe_template_singbox', label: 'SingBox', format: 'json' },
  { key: 'clash', configKey: 'subscribe_template_clash', label: 'Clash', format: 'yaml' },
  { key: 'clashmeta', configKey: 'subscribe_template_clashmeta', label: 'ClashMeta', format: 'yaml' },
  { key: 'stash', configKey: 'subscribe_template_stash', label: 'Stash', format: 'yaml' },
  { key: 'surge', configKey: 'subscribe_template_surge', label: 'Surge', format: 'yaml' },
  { key: 'surfboard', configKey: 'subscribe_template_surfboard', label: 'Surfboard', format: 'yaml' },
]);

const NODE_PROTOCOL_SETTINGS_TEMPLATES = Object.freeze({
  vmess: {
    tls: 1,
    network: 'ws',
    rules: null,
    network_settings: {
      header: {
        type: 'none',
        request: {
          path: ['/'],
          headers: { Host: null },
        },
      },
      path: '/',
      headers: { Host: null },
      serviceName: null,
      seed: null,
      host: null,
      mode: 'auto',
      extra: {},
    },
    tls_settings: {
      server_name: null,
      allow_insecure: false,
    },
  },
  vless: {
    tls: 0,
    flow: null,
    network: 'tcp',
    network_settings: {
      header: {
        type: 'none',
        request: {
          path: ['/'],
          headers: { Host: ['www.example.com'] },
        },
      },
      path: '/',
      headers: { Host: null },
      serviceName: null,
      seed: null,
      host: null,
      mode: 'auto',
      extra: {},
    },
    tls_settings: {
      server_name: null,
      allow_insecure: false,
    },
    reality_settings: {
      server_port: null,
      server_name: null,
      public_key: null,
      private_key: null,
      short_id: null,
      allow_insecure: false,
    },
  },
  trojan: {
    allow_insecure: false,
    server_name: null,
    network: 'tcp',
    network_settings: {
      path: '/',
      headers: { Host: null },
      serviceName: null,
    },
  },
  shadowsocks: {
    cipher: 'aes-128-gcm',
    obfs: null,
    obfs_settings: {
      host: null,
      path: null,
    },
    plugin: null,
    plugin_opts: null,
  },
  hysteria: {
    version: 1,
    bandwidth: { up: 100, down: 100 },
    tls: { server_name: null, allow_insecure: false },
    obfs: { open: false, type: 'salamander', password: null },
    hop_interval: null,
  },
  hysteria2: {
    version: 2,
    bandwidth: { up: 100, down: 100 },
    tls: { server_name: null, allow_insecure: false },
    obfs: { open: false, type: 'salamander', password: null },
    hop_interval: null,
  },
  tuic: {
    version: 5,
    congestion_control: 'cubic',
    alpn: ['h3'],
    udp_relay_mode: 'native',
    zero_rtt_handshake: false,
    heartbeat: '10s',
    tls: { server_name: null, allow_insecure: false },
  },
  anytls: {
    tls: { server_name: null, allow_insecure: false },
    padding_scheme: [
      'stop=8',
      '0=30-30',
      '1=100-400',
      '2=400-500,c,500-1000,c,500-1000,c,500-1000,c,500-1000',
      '3=9-9,500-1000',
      '4=500-1000',
      '5=500-1000',
      '6=500-1000',
      '7=500-1000',
    ],
  },
  socks: {
    tls: 0,
    tls_settings: { allow_insecure: false, server_name: null },
    udp_over_tcp: false,
  },
  http: {
    tls: 0,
    tls_settings: { allow_insecure: false, server_name: null },
    path: null,
    headers: null,
  },
  naive: {
    tls: 0,
    tls_settings: { allow_insecure: false, server_name: null },
  },
  mieru: {
    protocol: 0,
    transport: 'tcp',
    multiplexing: 'MULTIPLEXING_LOW',
  },
});

const NODE_SETTING_PATH_LABELS = Object.freeze({
  tls: 'TLS 开关',
  flow: '流控模式',
  network: '传输网络',
  allow_insecure: '允许不安全证书',
  server_name: '服务器名称',
  cipher: '加密方式',
  obfs: '混淆配置',
  plugin: '插件名称',
  plugin_opts: '插件参数',
  version: '协议版本',
  congestion_control: '拥塞控制',
  alpn: 'ALPN',
  udp_relay_mode: 'UDP 转发模式',
  protocol: '协议模式',
  transport: '传输类型',
  multiplexing: '多路复用级别',
  'network_settings.header.type': '请求头伪装类型',
  'network_settings.path': '路径',
  'network_settings.headers.host': 'Host',
  'network_settings.servicename': '服务名',
  'network_settings.seed': '种子',
  'network_settings.host': '主机名',
  'network_settings.mode': '模式',
  'tls_settings.server_name': 'TLS 服务器名称',
  'tls_settings.allow_insecure': 'TLS 跳过证书校验',
  'reality_settings.server_port': 'Reality 目标端口',
  'reality_settings.server_name': 'Reality 目标域名',
  'reality_settings.public_key': 'Reality 公钥',
  'reality_settings.private_key': 'Reality 私钥',
  'reality_settings.short_id': 'Reality Short ID',
  'reality_settings.allow_insecure': 'Reality 跳过证书校验',
  'obfs.open': '启用混淆',
  'obfs.type': '混淆类型',
  'obfs.password': '混淆密码',
  'bandwidth.up': '上行带宽 (Mbps)',
  'bandwidth.down': '下行带宽 (Mbps)',
  'hop_interval': '端口跳跃间隔',
  zero_rtt_handshake: '0-RTT 握手',
  heartbeat: '心跳间隔',
  'padding_scheme': '填充策略',
});

const NODE_SETTING_SEGMENT_LABELS = Object.freeze({
  tls: 'TLS',
  flow: '流控',
  network: '网络',
  rules: '规则',
  network_settings: '网络参数',
  tls_settings: 'TLS 参数',
  reality_settings: 'Reality 参数',
  allow_insecure: '跳过证书校验',
  server_name: '服务器名称',
  server_port: '服务器端口',
  public_key: '公钥',
  private_key: '私钥',
  short_id: 'Short ID',
  header: '请求头',
  type: '类型',
  request: '请求',
  response: '响应',
  path: '路径',
  headers: '请求头',
  host: 'Host',
  servicename: '服务名',
  seed: '种子',
  mode: '模式',
  extra: '扩展',
  obfs: '混淆',
  obfs_settings: '混淆参数',
  cipher: '加密方式',
  plugin: '插件',
  plugin_opts: '插件参数',
  version: '版本',
  bandwidth: '带宽',
  up: '上行',
  down: '下行',
  hop_interval: '跳跃间隔',
  zero_rtt_handshake: '0-RTT',
  heartbeat: '心跳',
  congestion_control: '拥塞控制',
  alpn: 'ALPN',
  udp_relay_mode: 'UDP 转发',
  padding_scheme: '填充策略',
  protocol: '协议',
  transport: '传输',
  multiplexing: '多路复用',
});

const NODE_SETTING_ENUM_OPTIONS = Object.freeze({
  tls: [
    { value: 0, label: '关闭' },
    { value: 1, label: '开启' },
    { value: 2, label: 'REALITY' },
  ],
  flow: [
    { value: '', label: '默认' },
    { value: 'xtls-rprx-vision', label: 'Vision' },
    { value: 'xtls-rprx-vision-udp443', label: 'Vision (UDP443)' },
  ],
  network: [
    { value: 'tcp', label: 'TCP' },
    { value: 'ws', label: 'WebSocket' },
    { value: 'grpc', label: 'gRPC' },
    { value: 'httpupgrade', label: 'HTTP Upgrade' },
    { value: 'http', label: 'HTTP' },
    { value: 'kcp', label: 'mKCP' },
    { value: 'quic', label: 'QUIC' },
  ],
  'network_settings.header.type': [
    { value: 'none', label: '无伪装' },
    { value: 'http', label: 'HTTP 伪装' },
  ],
  'network_settings.mode': [
    { value: 'auto', label: '自动' },
    { value: 'gun', label: 'Gun' },
    { value: 'multi', label: 'Multi' },
  ],
  cipher: [
    { value: 'aes-128-gcm', label: 'AES-128-GCM' },
    { value: 'aes-256-gcm', label: 'AES-256-GCM' },
    { value: 'chacha20-ietf-poly1305', label: 'ChaCha20-Poly1305' },
    { value: '2022-blake3-aes-128-gcm', label: '2022 AES-128-GCM' },
    { value: '2022-blake3-aes-256-gcm', label: '2022 AES-256-GCM' },
    { value: '2022-blake3-chacha20-poly1305', label: '2022 ChaCha20-Poly1305' },
  ],
  'obfs.type': [
    { value: 'salamander', label: 'Salamander' },
  ],
  congestion_control: [
    { value: 'cubic', label: 'Cubic' },
    { value: 'bbr', label: 'BBR' },
    { value: 'new_reno', label: 'New Reno' },
  ],
  udp_relay_mode: [
    { value: 'native', label: '原生' },
    { value: 'quic', label: 'QUIC' },
  ],
  protocol: [
    { value: 0, label: '默认' },
    { value: 1, label: '增强' },
  ],
  transport: [
    { value: 'tcp', label: 'TCP' },
    { value: 'udp', label: 'UDP' },
  ],
  multiplexing: [
    { value: 'MULTIPLEXING_OFF', label: '关闭' },
    { value: 'MULTIPLEXING_LOW', label: '低' },
    { value: 'MULTIPLEXING_MIDDLE', label: '中' },
    { value: 'MULTIPLEXING_HIGH', label: '高' },
  ],
});

const NODE_PROTOCOL_EDITOR_META = Object.freeze({
  vmess: {
    recommendedPaths: [
      'tls',
      'network',
      'tls_settings.server_name',
      'tls_settings.allow_insecure',
      'network_settings.path',
      'network_settings.headers.Host',
    ],
  },
  vless: {
    defaultMode: 'tls',
    recommendedPathsByMode: {
      tls: [
        'tls',
        'network',
        'flow',
        'tls_settings.server_name',
        'tls_settings.allow_insecure',
        'network_settings.path',
        'network_settings.headers.Host',
      ],
      reality: [
        'tls',
        'network',
        'flow',
        'reality_settings.server_port',
        'reality_settings.server_name',
        'reality_settings.public_key',
        'reality_settings.private_key',
        'reality_settings.short_id',
      ],
    },
    modePresets: {
      tls: {
        label: 'TLS',
        managedPaths: [
          'tls',
          'network',
          'tls_settings.server_name',
          'tls_settings.allow_insecure',
          'network_settings.path',
          'network_settings.headers.Host',
        ],
        settings: {
          tls: 1,
          network: 'ws',
          tls_settings: {
            server_name: '__HOST__',
            allow_insecure: false,
          },
          network_settings: {
            path: '/',
            headers: { Host: '__HOST__' },
          },
        },
      },
      reality: {
        label: 'REALITY',
        managedPaths: [
          'tls',
          'network',
          'flow',
          'reality_settings.server_port',
          'reality_settings.server_name',
          'reality_settings.public_key',
          'reality_settings.private_key',
          'reality_settings.short_id',
          'reality_settings.allow_insecure',
          'tls_settings.server_name',
          'tls_settings.allow_insecure',
          'network_settings.path',
          'network_settings.headers.Host',
        ],
        settings: {
          tls: 2,
          network: 'tcp',
          flow: null,
          tls_settings: {
            server_name: null,
            allow_insecure: false,
          },
          reality_settings: {
            server_port: 443,
            server_name: null,
            public_key: null,
            private_key: null,
            short_id: null,
            allow_insecure: false,
          },
          network_settings: {
            path: '/',
            headers: { Host: null },
          },
        },
      },
    },
  },
  trojan: {
    recommendedPaths: [
      'server_name',
      'allow_insecure',
      'network',
      'network_settings.path',
      'network_settings.headers.Host',
    ],
  },
  shadowsocks: {
    recommendedPaths: ['cipher', 'plugin', 'plugin_opts'],
  },
  hysteria: {
    recommendedPaths: [
      'version',
      'tls.server_name',
      'tls.allow_insecure',
      'bandwidth.up',
      'bandwidth.down',
      'obfs.open',
      'obfs.type',
      'obfs.password',
    ],
  },
  hysteria2: {
    recommendedPaths: [
      'version',
      'tls.server_name',
      'tls.allow_insecure',
      'bandwidth.up',
      'bandwidth.down',
      'obfs.open',
      'obfs.type',
      'obfs.password',
    ],
  },
  tuic: {
    recommendedPaths: [
      'version',
      'congestion_control',
      'tls.server_name',
      'tls.allow_insecure',
      'udp_relay_mode',
      'alpn',
      'zero_rtt_handshake',
      'heartbeat',
    ],
  },
  anytls: {
    recommendedPaths: ['tls.server_name', 'tls.allow_insecure'],
  },
  socks: {
    recommendedPaths: ['tls', 'tls_settings.server_name', 'tls_settings.allow_insecure', 'udp_over_tcp'],
  },
  http: {
    recommendedPaths: ['tls', 'tls_settings.server_name', 'tls_settings.allow_insecure', 'path'],
  },
  naive: {
    recommendedPaths: ['tls', 'tls_settings.server_name', 'tls_settings.allow_insecure'],
  },
  mieru: {
    recommendedPaths: ['transport', 'multiplexing', 'protocol'],
  },
});

let nodeProtocolOptionsCache = null;
let nodeProtocolTemplateCache = {};

function html(strings, ...values) {
  return strings.map((s, i) => s + (values[i] ?? '')).join('');
}

function setView(markup) {
  VIEW.innerHTML = markup;
}

function setContainerWideMode(enabled) {
  if (!APP_CONTAINER) return;
  APP_CONTAINER.classList.toggle('wide-mode', Boolean(enabled));
}

function setAppChromeMode(mode) {
  const enabled = Boolean(mode);
  document.body.classList.toggle('command-center-mode', enabled);
  if (APP_ROOT) APP_ROOT.classList.toggle('command-center-app', enabled);
  if (APP_TOPBAR) APP_TOPBAR.classList.toggle('hidden', enabled);
  if (APP_FOOTER) APP_FOOTER.classList.toggle('hidden', enabled);
}

function qs(sel, root = document) { return root.querySelector(sel); }
function qsa(sel, root = document) { return Array.from(root.querySelectorAll(sel)); }
function escapeHtml(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function normalizeFieldErrorMessages(value) {
  if (Array.isArray(value)) {
    return value
      .map((item) => String(item ?? '').trim())
      .filter(Boolean);
  }

  const text = String(value ?? '').trim();
  return text ? [text] : [];
}

function setInlineFormError(container, message = '') {
  const target = typeof container === 'string' ? qs(container) : container;
  if (!target) return;

  const text = String(message || '').trim();
  target.innerHTML = text ? `<div class="notice error">${escapeHtml(text)}</div>` : '';
}

function clearFormValidationState(rootNode) {
  const root = typeof rootNode === 'string' ? qs(rootNode) : rootNode;
  if (!root) return;

  qsa('[data-validation-invalid="1"]', root).forEach((node) => {
    node.removeAttribute('data-validation-invalid');
    node.removeAttribute('aria-invalid');
    node.style.removeProperty('border-color');
    node.style.removeProperty('box-shadow');
    node.style.removeProperty('background-color');
    node.style.removeProperty('outline');
    node.style.removeProperty('outline-offset');
    node.style.removeProperty('border-radius');
  });

  qsa('[data-validation-error="1"]', root).forEach((node) => node.remove());
}

function resolveValidationFieldTarget(rootNode, fieldMap, fieldKey) {
  const root = typeof rootNode === 'string' ? qs(rootNode) : rootNode;
  if (!root || !fieldMap || !fieldKey) return null;

  let cursor = String(fieldKey).trim();
  while (cursor) {
    const selector = fieldMap[cursor];
    if (selector) {
      const target = qs(selector, root);
      if (target) return target;
    }

    const lastDot = cursor.lastIndexOf('.');
    if (lastDot < 0) break;
    cursor = cursor.slice(0, lastDot);
  }

  return null;
}

function markValidationError(target, messages) {
  if (!target) return null;

  const field = target.closest('.field') || target.parentElement || target;
  const isFormControl = Boolean(target.matches && target.matches('input, select, textarea'));
  const highlightTarget = isFormControl ? target : target;
  const focusTarget = isFormControl ? target : (qs('input, select, textarea', field) || target);

  highlightTarget.setAttribute('data-validation-invalid', '1');
  highlightTarget.setAttribute('aria-invalid', 'true');

  if (isFormControl) {
    highlightTarget.style.borderColor = '#d64242';
    highlightTarget.style.boxShadow = '0 0 0 3px rgba(214, 66, 66, 0.14)';
    highlightTarget.style.backgroundColor = '#fff7f7';
  } else {
    highlightTarget.style.outline = '2px solid rgba(214, 66, 66, 0.55)';
    highlightTarget.style.outlineOffset = '6px';
    highlightTarget.style.borderRadius = '12px';
  }

  let errorNode = qs('[data-validation-error="1"]', field);
  if (!errorNode) {
    errorNode = document.createElement('div');
    errorNode.setAttribute('data-validation-error', '1');
    errorNode.style.marginTop = '6px';
    errorNode.style.color = '#d64242';
    errorNode.style.fontSize = '12px';
    errorNode.style.lineHeight = '1.5';
    field.appendChild(errorNode);
  }
  errorNode.textContent = messages.join(' ');

  return focusTarget;
}

function presentFormSubmitError({ rootNode, errorBox, fieldMap, error, fallbackMessage = '' }) {
  const root = typeof rootNode === 'string' ? qs(rootNode) : rootNode;
  if (!root) return;

  clearFormValidationState(root);

  const validationErrors = error?.payload?.errors;
  const isValidationError = error?.status === 422 && validationErrors && typeof validationErrors === 'object';
  if (!isValidationError) {
    const message = fallbackMessage || error?.message || '提交失败';
    setInlineFormError(errorBox, message);
    return;
  }

  let firstMessage = '';
  let firstTarget = null;

  Object.entries(validationErrors).forEach(([fieldKey, rawMessages]) => {
    const messages = normalizeFieldErrorMessages(rawMessages);
    if (!messages.length) return;

    if (!firstMessage) {
      firstMessage = messages[0];
    }

    const target = resolveValidationFieldTarget(root, fieldMap, fieldKey);
    if (!target) return;

    const focusTarget = markValidationError(target, messages);
    if (!firstTarget && focusTarget) {
      firstTarget = focusTarget;
    }
  });

  setInlineFormError(errorBox, firstMessage || '请检查标红项后再提交。');

  if (!firstTarget) return;

  requestAnimationFrame(() => {
    firstTarget.scrollIntoView({ behavior: 'smooth', block: 'center' });
    if (typeof firstTarget.focus === 'function') {
      try {
        firstTarget.focus({ preventScroll: true });
      } catch (_) {
        firstTarget.focus();
      }
    }
  });
}

function bindFormValidationAutoClear(rootNode, errorBox) {
  const root = typeof rootNode === 'string' ? qs(rootNode) : rootNode;
  if (!root) return;

  const clearHandler = (event) => {
    const field = event.target.closest('.field');
    if (field) {
      clearFormValidationState(field);
    }

    const protocolGrid = event.target.closest('.protocol-settings-grid');
    if (protocolGrid) {
      const protocolField = protocolGrid.closest('.field');
      if (protocolField) {
        clearFormValidationState(protocolField);
      }
    }

    if (!qsa('[data-validation-error="1"]', root).length) {
      setInlineFormError(errorBox, '');
    }
  };

  qsa('input, select, textarea', root).forEach((input) => {
    const primaryEvent = input.tagName === 'SELECT' ? 'change' : 'input';
    input.addEventListener(primaryEvent, clearHandler);
    if (primaryEvent !== 'change') {
      input.addEventListener('change', clearHandler);
    }
  });
}

function getSubscribeTemplateEditor(type) {
  const normalized = String(type || '').toLowerCase();
  return SUBSCRIBE_TEMPLATE_EDITORS.find((item) => item.key === normalized) || SUBSCRIBE_TEMPLATE_EDITORS[0];
}

function mountAdminSidebar(initialCardId = '') {
  const rootCards = qsa('.card', VIEW).filter((card) => qs('h2', card));
  if (!rootCards.length) {
    return { getCurrent: () => 'all' };
  }

  const idByTitle = (title, idx) => {
    const t = String(title || '');
    if (t.includes('同用户异 IP 限制')) return 'ip-limits';
    if (t.includes('API Key 系统统计')) return 'api-key-stats';
    if (t.includes('登录与安全设置')) return 'security';
    if (t.includes('OAuth2 登录设置')) return 'oauth2';
    if (t.includes('客户端版本与下载地址')) return 'client-downloads';
    if (t.includes('记录保留与订阅凭据')) return 'retention';
    if (t.includes('代理订阅模板')) return 'proxy-templates';
    if (t.includes('已迁移模块')) return 'migrated-overview';
    if (t.includes('主题与订阅套餐')) return 'themes-plans';
    if (t.includes('支付与公告')) return 'payments-notices';
    if (t.includes('工单与优惠券')) return 'tickets-coupons';
    if (t.includes('礼品卡与插件')) return 'gift-plugins';
    if (t.includes('系统状态与日志')) return 'system-status';
    if (t.includes('流量重置管理')) return 'traffic-reset';
    if (t.includes('用户组限制')) return 'group-limits';
    if (t.includes('用户个人限制')) return 'user-limits';
    if (t.includes('赞助收款配置')) return 'sponsor-config';
    if (t.includes('API Key 搜索与用户操作')) return 'api-key-ops';
    if (t.includes('节点套餐（全局）')) return 'global-node-plans';
    if (t.includes('退款（全局）')) return 'global-refunds';
    return `module-${idx + 1}`;
  };

  const usedIds = new Set();
  const cards = rootCards.map((card, idx) => {
    const title = String(qs('h2', card)?.textContent || `模块 ${idx + 1}`).trim();
    let id = idByTitle(title, idx);
    let seed = 2;
    while (usedIds.has(id)) {
      id = `${id}-${seed}`;
      seed += 1;
    }
    usedIds.add(id);
    card.dataset.adminCardId = id;
    card.id = `admin-card-${id}`;
    return { id, title, card };
  });

  const content = document.createElement('div');
  content.className = 'admin-main';
  while (VIEW.firstChild) {
    content.appendChild(VIEW.firstChild);
  }

  const shell = document.createElement('div');
  shell.className = 'admin-shell';
  const nav = document.createElement('aside');
  nav.className = 'card admin-sidebar';
  nav.innerHTML = html`
    <h2>超管导航</h2>
    <div class="muted">点击切换对应模块，避免页面平铺过长。</div>
    <div class="admin-nav-list" style="margin-top:10px;">
      <button class="btn small admin-nav-btn" data-admin-nav="all">显示全部</button>
      ${cards.map((item) => `<button class="btn small admin-nav-btn" data-admin-nav="${item.id}">${escapeHtml(item.title)}</button>`).join('')}
    </div>
  `;

  shell.appendChild(nav);
  shell.appendChild(content);
  VIEW.appendChild(shell);

  const navButtons = qsa('button[data-admin-nav]', nav);
  let current = 'all';
  const fallback = localStorage.getItem('maintainable_admin_active_card') || 'all';
  const desired = String(initialCardId || '').trim() || fallback;
  const hasDesired = desired === 'all' || cards.some((item) => item.id === desired);

  const applyCard = (cardId, updateHash = true) => {
    const next = cardId === 'all' || cards.some((item) => item.id === cardId) ? cardId : 'all';
    current = next;
    localStorage.setItem('maintainable_admin_active_card', current);

    cards.forEach((item) => {
      item.card.classList.toggle('admin-card-hidden', current !== 'all' && item.id !== current);
    });
    navButtons.forEach((btn) => {
      btn.classList.toggle('active', btn.getAttribute('data-admin-nav') === current);
    });

    if (updateHash) {
      const targetHash = current === 'all' ? '#/admin' : `#/admin/${encodeURIComponent(current)}`;
      if (location.hash !== targetHash) {
        history.replaceState(null, '', targetHash);
      }
    }
  };

  navButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
      applyCard(String(btn.getAttribute('data-admin-nav') || 'all'));
    });
  });

  applyCard(hasDesired ? desired : 'all', false);

  return {
    getCurrent: () => current,
  };
}

function protocolDisplayName(protocol, protocolMap = {}) {
  const key = String(protocol || '').toLowerCase();
  return protocolMap[key] || NODE_PROTOCOL_LABELS[key] || (key ? key.toUpperCase() : '-');
}

function isV2bxDeployableProtocol(protocol) {
  return V2BX_DEPLOYABLE_PROTOCOLS.has(String(protocol || '').toLowerCase());
}

function buildProtocolOptionHtml(protocolOptions, selectedValue = '') {
  const selectedKey = String(selectedValue || '').toLowerCase();
  return protocolOptions
    .map((item) => {
      const value = String(item?.value || '').toLowerCase();
      const label = String(item?.label || protocolDisplayName(value));
      const selected = value === selectedKey ? ' selected' : '';
      return `<option value="${escapeHtml(value)}"${selected}>${escapeHtml(label)}</option>`;
    })
    .join('');
}

function buildTrustLevelOptionHtml(selectedValue = 0, includeEmpty = false) {
  const selected = includeEmpty && (selectedValue === '' || selectedValue === null || selectedValue === undefined)
    ? ''
    : String(Number(selectedValue) || 0);
  const rows = [];
  if (includeEmpty) {
    rows.push(`<option value="" ${selected === '' ? 'selected' : ''}>不限制</option>`);
  }
  TRUST_LEVEL_OPTIONS.forEach((level) => {
    const value = String(level);
    const label = `等级 ${level}`;
    rows.push(`<option value="${value}" ${selected === value ? 'selected' : ''}>${label}</option>`);
  });
  return rows.join('');
}

function deepClone(value) {
  if (value === undefined) return undefined;
  return JSON.parse(JSON.stringify(value));
}

function safeDecodeURIComponent(value) {
  const text = String(value || '');
  if (!text) return '';
  try {
    return decodeURIComponent(text);
  } catch (_) {
    return text;
  }
}

function parseHashRoute(hashValue = location.hash || '#/dashboard') {
  const rawHash = String(hashValue || '#/dashboard').trim() || '#/dashboard';
  const withoutHash = rawHash.startsWith('#') ? rawHash.slice(1) : rawHash;
  const [pathPartRaw, queryString = ''] = withoutHash.split('?');
  const normalizedPath = (() => {
    const text = String(pathPartRaw || '/dashboard').trim();
    if (!text) return '/dashboard';
    return text.startsWith('/') ? text : `/${text}`;
  })();
  const params = new URLSearchParams(queryString);
  const query = {};
  params.forEach((value, key) => {
    query[key] = value;
  });

  return {
    rawHash,
    path: `#${normalizedPath}`,
    pathWithoutHash: normalizedPath,
    query,
    queryString: params.toString(),
    segments: normalizedPath
      .replace(/^\/+/, '')
      .split('/')
      .filter(Boolean)
      .map((segment) => safeDecodeURIComponent(segment)),
  };
}

function buildHashPath(path = '/dashboard', query = {}) {
  const normalizedPath = (() => {
    const text = String(path || '/dashboard').trim();
    const cleaned = text.startsWith('#') ? text.slice(1) : text;
    if (!cleaned) return '/dashboard';
    return cleaned.startsWith('/') ? cleaned : `/${cleaned}`;
  })();

  const params = new URLSearchParams();
  Object.entries(query || {}).forEach(([key, value]) => {
    if (value === null || value === undefined) return;
    const text = String(value).trim();
    if (!text) return;
    params.set(key, text);
  });

  const queryText = params.toString();
  return `#${normalizedPath}${queryText ? `?${queryText}` : ''}`;
}

function getCurrentAppBaseUrl() {
  return `${location.origin}${location.pathname}${location.search}`;
}

function buildAppHashUrl(path = '/dashboard', query = {}) {
  return `${getCurrentAppBaseUrl()}${buildHashPath(path, query)}`;
}

function buildOAuthLinuxDoUrl(inviteCode = '') {
  const url = new URL('/api/v1/passport/oauth2/linux-do/redirect', location.origin);
  const code = String(inviteCode || '').trim();
  if (code) {
    url.searchParams.set('invite_code', code);
  }
  return url.toString();
}

function mergeTemplatePreferDefined(primary, fallback) {
  if (primary === null || primary === undefined) {
    return deepClone(fallback);
  }

  if (Array.isArray(primary)) {
    return deepClone(primary);
  }

  if (typeof primary !== 'object') {
    return primary;
  }

  const fallbackObj = (fallback && typeof fallback === 'object' && !Array.isArray(fallback)) ? fallback : {};
  const out = {};
  const keys = new Set([...Object.keys(fallbackObj), ...Object.keys(primary)]);
  keys.forEach((key) => {
    const pVal = primary[key];
    const fVal = fallbackObj[key];
    if (pVal === null || pVal === undefined) {
      out[key] = deepClone(fVal);
      return;
    }
    if (typeof pVal === 'object' && !Array.isArray(pVal)) {
      out[key] = mergeTemplatePreferDefined(pVal, fVal);
      return;
    }
    out[key] = deepClone(pVal);
  });
  return out;
}

async function loadNodeProtocolOptions(forceReload = false) {
  if (!forceReload && Array.isArray(nodeProtocolOptionsCache) && nodeProtocolOptionsCache.length) {
    return nodeProtocolOptionsCache;
  }

  try {
    const res = await apiFetch('/api/v1/user/server-nodes/protocols');
    const rows = Array.isArray(res?.data) ? res.data : [];
    const templateMap = {};
    const normalized = rows
      .map((row) => {
        const value = String(row?.value || '').toLowerCase();
        if (!value) return null;
        const template = row?.template;
        if (template && typeof template === 'object' && !Array.isArray(template)) {
          templateMap[value] = template;
        }
        return {
          value,
          label: String(row?.label || protocolDisplayName(value)),
        };
      })
      .filter(Boolean);

    if (normalized.length) {
      nodeProtocolOptionsCache = normalized;
      nodeProtocolTemplateCache = templateMap;
      return normalized;
    }
  } catch (_) {}

  nodeProtocolOptionsCache = DEFAULT_NODE_PROTOCOL_OPTIONS;
  nodeProtocolTemplateCache = {};
  return DEFAULT_NODE_PROTOCOL_OPTIONS;
}

function getProtocolSettingsTemplate(protocol) {
  const key = String(protocol || '').toLowerCase();
  const localTpl = NODE_PROTOCOL_SETTINGS_TEMPLATES[key] || {};
  const remoteTemplate = nodeProtocolTemplateCache?.[key];
  if (remoteTemplate && typeof remoteTemplate === 'object' && !Array.isArray(remoteTemplate)) {
    return mergeTemplatePreferDefined(remoteTemplate, localTpl);
  }
  return localTpl ? deepClone(localTpl) : {};
}

function prettyJson(input) {
  return JSON.stringify(input ?? {}, null, 2);
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function mergeProtocolSettingsForEditor(protocol, currentSettings = null) {
  const key = String(protocol || '').toLowerCase();
  const template = getProtocolSettingsTemplate(protocol);
  if (isPlainObject(currentSettings)) {
    return mergeTemplatePreferDefined(currentSettings, template);
  }
  const defaults = isPlainObject(template) ? deepClone(template) : {};
  const defaultMode = NODE_PROTOCOL_EDITOR_META[key]?.defaultMode;
  if (defaultMode) {
    return applyProtocolEditorModePreset(protocol, defaults, defaultMode);
  }
  return defaults;
}

function detectArrayElementType(input) {
  const arr = Array.isArray(input) ? input : [];
  const values = arr.filter((item) => item !== null && item !== undefined);
  if (!values.length) return 'string';
  if (values.every((item) => typeof item === 'number')) return 'number';
  if (values.every((item) => typeof item === 'boolean')) return 'boolean';
  return 'string';
}

function settingPathToLabel(path) {
  const normalized = String(path || '').trim().toLowerCase();
  if (NODE_SETTING_PATH_LABELS[normalized]) {
    return NODE_SETTING_PATH_LABELS[normalized];
  }

  const parts = normalized
    .split('.')
    .map((item) => item.trim())
    .filter(Boolean)
    .map((item) => NODE_SETTING_SEGMENT_LABELS[item] || item.toUpperCase());

  return parts.length ? parts.join(' / ') : '参数';
}

function normalizeEnumValueForCompare(value) {
  if (value === null || value === undefined) return '';
  return String(value);
}

function getSettingEnumOptions(path, currentValue) {
  const normalizedPath = String(path || '').trim().toLowerCase();
  const last = normalizedPath.split('.').pop() || normalizedPath;
  const options = NODE_SETTING_ENUM_OPTIONS[normalizedPath] || NODE_SETTING_ENUM_OPTIONS[last];
  if (!Array.isArray(options) || !options.length) return null;

  const current = normalizeEnumValueForCompare(currentValue);
  const hasCurrent = options.some((item) => normalizeEnumValueForCompare(item?.value) === current);
  if (hasCurrent || current === '') return options;

  return [
    ...options,
    { value: currentValue, label: `自定义值 (${current})` },
  ];
}

function getValueByPath(source, path) {
  const segments = String(path || '')
    .split('.')
    .map((item) => item.trim())
    .filter(Boolean);
  let cursor = source;
  for (const segment of segments) {
    if (!cursor || typeof cursor !== 'object' || !(segment in cursor)) {
      return undefined;
    }
    cursor = cursor[segment];
  }
  return cursor;
}

function deleteValueByPath(target, path) {
  const segments = String(path || '')
    .split('.')
    .map((item) => item.trim())
    .filter(Boolean);
  if (!segments.length || !isPlainObject(target)) return;

  const walk = (cursor, index) => {
    const key = segments[index];
    if (!cursor || typeof cursor !== 'object' || !(key in cursor)) return;
    if (index === segments.length - 1) {
      delete cursor[key];
      return;
    }
    walk(cursor[key], index + 1);
    if (isPlainObject(cursor[key]) && !Object.keys(cursor[key]).length) {
      delete cursor[key];
    }
  };

  walk(target, 0);
}

function collectLeafSettingPaths(settings, pathPrefix = '') {
  if (!isPlainObject(settings)) {
    return pathPrefix ? [pathPrefix] : [];
  }

  return Object.entries(settings).flatMap(([key, value]) => {
    const path = pathPrefix ? `${pathPrefix}.${key}` : key;
    if (isPlainObject(value)) {
      return collectLeafSettingPaths(value, path);
    }
    return [path];
  });
}

function renderSingleProtocolSettingField(path, value) {
  if (value === undefined) return '';

  const inputId = `setting_${path.replace(/[^a-zA-Z0-9_]+/g, '_')}`;
  const label = settingPathToLabel(path);
  const enumOptions = getSettingEnumOptions(path, value);

  if (Array.isArray(value)) {
    const elementType = detectArrayElementType(value);
    const textValue = value.map((item) => String(item ?? '')).join('\n');
    return html`
      <div class="field">
        <label for="${escapeHtml(inputId)}">${escapeHtml(label)}（数组）</label>
        <textarea
          id="${escapeHtml(inputId)}"
          rows="4"
          data-setting-path="${escapeHtml(path)}"
          data-setting-type="array"
          data-setting-array-type="${escapeHtml(elementType)}"
          placeholder="每行一项"
        >${escapeHtml(textValue)}</textarea>
        <div class="muted">每行一项，留空表示空数组。</div>
      </div>
    `;
  }

  if (typeof value === 'boolean') {
    return html`
      <div class="field">
        <label for="${escapeHtml(inputId)}">${escapeHtml(label)}</label>
        <select
          id="${escapeHtml(inputId)}"
          data-setting-path="${escapeHtml(path)}"
          data-setting-type="boolean"
        >
          <option value="1" ${value ? 'selected' : ''}>是</option>
          <option value="0" ${value ? '' : 'selected'}>否</option>
        </select>
      </div>
    `;
  }

  if (enumOptions && (typeof value === 'number' || typeof value === 'string' || value === null)) {
    const type = typeof value === 'number' ? 'number' : 'string';
    const selected = normalizeEnumValueForCompare(value);
    const nullable = value === null ? '1' : '0';
    return html`
      <div class="field">
        <label for="${escapeHtml(inputId)}">${escapeHtml(label)}</label>
        <select
          id="${escapeHtml(inputId)}"
          data-setting-path="${escapeHtml(path)}"
          data-setting-type="${escapeHtml(type)}"
          data-setting-nullable="${nullable}"
        >
          ${enumOptions.map((item) => {
            const optionValue = normalizeEnumValueForCompare(item?.value);
            const optionLabel = item?.label ?? optionValue;
            return `<option value="${escapeHtml(optionValue)}" ${optionValue === selected ? 'selected' : ''}>${escapeHtml(optionLabel)}</option>`;
          }).join('')}
        </select>
      </div>
    `;
  }

  if (typeof value === 'number') {
    return html`
      <div class="field">
        <label for="${escapeHtml(inputId)}">${escapeHtml(label)}</label>
        <input
          id="${escapeHtml(inputId)}"
          type="number"
          step="0.01"
          value="${escapeHtml(String(value))}"
          data-setting-path="${escapeHtml(path)}"
          data-setting-type="number"
        >
      </div>
    `;
  }

  const nullable = value === null ? '1' : '0';
  const textValue = value === null || value === undefined ? '' : String(value);
  return html`
    <div class="field">
      <label for="${escapeHtml(inputId)}">${escapeHtml(label)}</label>
      <input
        id="${escapeHtml(inputId)}"
        value="${escapeHtml(textValue)}"
        data-setting-path="${escapeHtml(path)}"
        data-setting-type="string"
        data-setting-nullable="${nullable}"
        placeholder="${value === null ? '留空表示空值' : ''}"
      >
    </div>
  `;
}

function renderSelectedProtocolSettingsFields(settings, paths) {
  const uniquePaths = Array.from(new Set(asArray(paths).map((item) => String(item || '').trim()).filter(Boolean)));
  const markup = uniquePaths
    .map((path) => renderSingleProtocolSettingField(path, getValueByPath(settings, path)))
    .filter(Boolean)
    .join('');

  return markup || '<div class="muted">当前协议没有推荐参数。</div>';
}

function replaceProtocolPresetPlaceholders(value, context = {}) {
  if (Array.isArray(value)) {
    return value.map((item) => replaceProtocolPresetPlaceholders(item, context));
  }
  if (isPlainObject(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, replaceProtocolPresetPlaceholders(item, context)])
    );
  }
  if (value === '__HOST__') {
    return String(context.host || '').trim() || null;
  }
  return value;
}

function resolveProtocolEditorMode(protocol, settings) {
  const key = String(protocol || '').toLowerCase();
  if (key !== 'vless') return '';
  return Number(getValueByPath(settings, 'tls') || 0) === 2 ? 'reality' : 'tls';
}

function applyProtocolEditorModePreset(protocol, settings, mode, context = {}) {
  const meta = NODE_PROTOCOL_EDITOR_META[String(protocol || '').toLowerCase()];
  const preset = meta?.modePresets?.[String(mode || '').toLowerCase()];
  if (!preset) {
    return isPlainObject(settings) ? deepClone(settings) : {};
  }

  const next = isPlainObject(settings) ? deepClone(settings) : {};
  asArray(preset.managedPaths).forEach((path) => {
    deleteValueByPath(next, path);
  });
  const resolvedPreset = replaceProtocolPresetPlaceholders(deepClone(preset.settings || {}), context);
  return mergeTemplatePreferDefined(resolvedPreset, next);
}

function renderProtocolSettingsEditor(protocol, settings, context = {}) {
  const key = String(protocol || '').toLowerCase();
  const meta = NODE_PROTOCOL_EDITOR_META[key] || {};
  const mode = resolveProtocolEditorMode(protocol, settings);
  const recommendedPaths = mode && meta.recommendedPathsByMode
    ? (meta.recommendedPathsByMode[mode] || [])
    : (meta.recommendedPaths || []);

  const advancedSettings = isPlainObject(settings) ? deepClone(settings) : {};
  asArray(recommendedPaths).forEach((path) => {
    deleteValueByPath(advancedSettings, path);
  });
  const advancedLeafPaths = collectLeafSettingPaths(advancedSettings);

  return html`
    ${meta.modePresets
      ? `<div class="field" style="grid-column: 1 / -1;">
          <label>推荐模式</label>
          <select data-protocol-mode-select>
            ${Object.entries(meta.modePresets).map(([presetKey, preset]) => `
              <option value="${escapeHtml(presetKey)}" ${presetKey === mode ? 'selected' : ''}>${escapeHtml(preset?.label || presetKey)}</option>
            `).join('')}
          </select>
          <div class="muted">切换后会自动替换常见 TLS / REALITY 参数，其他高级字段会保留。</div>
        </div>`
      : ''}
    <div class="field" style="grid-column: 1 / -1;">
      <label>推荐参数</label>
      <div class="grid cols-2" style="margin-top:8px;">
        ${renderSelectedProtocolSettingsFields(settings, recommendedPaths)}
      </div>
    </div>
    ${advancedLeafPaths.length
      ? `<div class="field" style="grid-column: 1 / -1;">
          <details>
            <summary>高级设置（大多数情况无需修改）</summary>
            <div class="grid cols-2" style="margin-top:10px;">
              ${renderProtocolSettingsFields(advancedSettings)}
            </div>
          </details>
        </div>`
      : ''}
  `;
}

function renderProtocolSettingsFields(settings, pathPrefix = '') {
  if (!isPlainObject(settings)) {
    return '<div class="notice">当前协议没有可配置参数。</div>';
  }

  const rows = Object.entries(settings).map(([key, value]) => {
    const path = pathPrefix ? `${pathPrefix}.${key}` : key;

    if (isPlainObject(value)) {
      return html`
        <div class="settings-group">
          <div class="settings-group-title">${escapeHtml(settingPathToLabel(path))}</div>
          <div class="grid cols-2" style="margin-top:8px;">
            ${renderProtocolSettingsFields(value, path)}
          </div>
        </div>
      `;
    }
    return renderSingleProtocolSettingField(path, value);
  });

  return rows.join('');
}

function setObjectByPath(target, path, value) {
  const segments = String(path || '')
    .split('.')
    .map((item) => item.trim())
    .filter(Boolean);
  if (!segments.length) return;

  let cursor = target;
  for (let i = 0; i < segments.length; i += 1) {
    const segment = segments[i];
    const isLast = i === segments.length - 1;
    if (isLast) {
      cursor[segment] = value;
      return;
    }

    if (!isPlainObject(cursor[segment])) {
      cursor[segment] = {};
    }
    cursor = cursor[segment];
  }
}

function parseArrayFieldLines(rawText, elementType) {
  const lines = String(rawText || '')
    .split('\n')
    .map((item) => item.trim())
    .filter((item) => item !== '');

  if (!lines.length) return [];

  if (elementType === 'number') {
    return lines.map((item) => {
      const parsed = Number(item);
      if (!Number.isFinite(parsed)) {
        throw new Error(`数组项 "${item}" 不是合法数字`);
      }
      return parsed;
    });
  }

  if (elementType === 'boolean') {
    return lines.map((item) => {
      const normalized = item.toLowerCase();
      if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') return true;
      if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') return false;
      throw new Error(`数组项 "${item}" 不是合法布尔值`);
    });
  }

  return lines;
}

function collectProtocolSettingsFromForm(rootNode) {
  const root = typeof rootNode === 'string' ? qs(rootNode) : rootNode;
  if (!root) return {};

  const result = {};
  qsa('[data-setting-path]', root).forEach((input) => {
    const path = input.getAttribute('data-setting-path') || '';
    const type = input.getAttribute('data-setting-type') || 'string';
    const raw = input.value;

    let parsedValue = null;
    if (type === 'boolean') {
      parsedValue = String(raw) === '1';
    } else if (type === 'number') {
      const text = String(raw || '').trim();
      if (!text) {
        parsedValue = 0;
      } else {
        const value = Number(text);
        if (!Number.isFinite(value)) {
          throw new Error(`${settingPathToLabel(path)} 不是合法数字`);
        }
        parsedValue = value;
      }
    } else if (type === 'array') {
      parsedValue = parseArrayFieldLines(raw, input.getAttribute('data-setting-array-type') || 'string');
    } else {
      const text = String(raw ?? '').trim();
      const nullable = input.getAttribute('data-setting-nullable') === '1';
      parsedValue = (nullable && text === '') ? null : text;
    }

    setObjectByPath(result, path, parsedValue);
  });

  return result;
}

function generateRandomNodePort(min = 50000, max = 65535) {
  const lo = Math.max(1, Number(min) || 50000);
  const hi = Math.min(65535, Number(max) || 65535);
  if (lo >= hi) return lo;

  const span = hi - lo + 1;
  if (window.crypto && window.crypto.getRandomValues) {
    const bytes = new Uint32Array(1);
    window.crypto.getRandomValues(bytes);
    return lo + (bytes[0] % span);
  }

  return lo + Math.floor(Math.random() * span);
}

function setActiveNav() {
  const route = parseHashRoute(location.hash || '#/dashboard');
  const base = route.segments[0] || 'dashboard';
  NAV_LINKS.forEach((link) => {
    if (link.id === 'logoutBtn') return;
    const href = link.getAttribute('href') || '';
    const linkBase = parseHashRoute(href || '#/dashboard').segments[0] || 'dashboard';
    link.classList.toggle('active', linkBase === base);
  });
}

function clearAuthState() {
  store.auth = '';
  store.legacyToken = '';
  store.me = null;
}

function setPostLoginRedirect(hashValue) {
  const target = String(hashValue || '').trim();
  if (!target || !target.startsWith('#/')) return;
  const route = parseHashRoute(target);
  if ((route.segments[0] || '') === 'login') return;
  try {
    sessionStorage.setItem('post_login_hash', target);
  } catch (_) {
    // ignore sessionStorage failures
  }
}

function consumePostLoginRedirect() {
  try {
    const target = String(sessionStorage.getItem('post_login_hash') || '').trim();
    if (target) sessionStorage.removeItem('post_login_hash');
    if (!target || !target.startsWith('#/')) return '';
    const route = parseHashRoute(target);
    if ((route.segments[0] || '') === 'login') return '';
    return target;
  } catch (_) {
    return '';
  }
}

function syncAuthNav() {
  const authed = Boolean(store.auth);
  AUTH_ONLY_LINKS.forEach((link) => {
    if (!authed) {
      link.classList.add('hidden');
      link.hidden = true;
      return;
    }

    // 争议退款投票：由超管开关控制，关闭时不在顶部栏目渲染入口
    if (String(link.getAttribute('href') || '') === '#/refund-votes') {
      const disputeEnabled = store.me?.refund_dispute_enable === true || Number(store.me?.refund_dispute_enable) === 1;
      link.classList.toggle('hidden', !disputeEnabled);
      link.hidden = !disputeEnabled;
      return;
    }

    if (link.getAttribute('data-role') === 'super-admin') {
      const isSuperAdmin = store.me?.is_super_admin === true || Number(store.me?.is_super_admin) === 1;
      link.classList.toggle('hidden', !isSuperAdmin);
      link.hidden = !isSuperAdmin;
      return;
    }

    link.classList.remove('hidden');
    link.hidden = false;
  });
  const logoutLabel = logoutBtn.querySelector('[data-nav-label]');
  if (logoutLabel) logoutLabel.textContent = authed ? '退出' : '登录';
  else logoutBtn.textContent = authed ? '退出' : '登录';
}

function apiHeaders(extra = {}) {
  const headers = { 'Content-Type': 'application/json', ...extra };
  if (store.auth) headers['Authorization'] = store.auth;
  const adminPath = String(store.me?.secure_path || '').trim();
  if (/^[A-Za-z0-9][A-Za-z0-9_-]{7,63}$/.test(adminPath)) {
    headers['X-Admin-Path'] = adminPath;
  }
  return headers;
}

async function apiFetch(path, { method = 'GET', body, headers } = {}) {
  const res = await fetch(path, {
    method,
    headers: apiHeaders(headers),
    body: body !== undefined ? JSON.stringify(body) : undefined
  });

  const text = await res.text();
  let json = null;
  try { json = text ? JSON.parse(text) : null; } catch { json = null; }

  if (res.status === 401) {
    const err = new Error('登录已失效，请重新登录');
    err.status = res.status;
    err.payload = json;
    err.handled = true;
    clearAuthState();
    syncAuthNav();
    setPostLoginRedirect(location.hash || '#/dashboard');
    if ((parseHashRoute(location.hash || '#/login').segments[0] || '') !== 'login') {
      location.hash = '#/login';
    } else {
      const route = parseHashRoute(location.hash || '#/login');
      renderLogin(
        err.message,
        route.query.register === '1' ? 'register' : 'login',
        { inviteCode: route.query.invite_code || '' }
      );
    }
    throw err;
  }

  if (!res.ok) {
    const message = json?.message || json?.error || `HTTP ${res.status}`;
    const err = new Error(message);
    err.status = res.status;
    err.payload = json;
    throw err;
  }
  return json;
}

async function loadGuestCommConfig(force = false) {
  if (!force && guestCommConfigCache) return guestCommConfigCache;
  if (!force && guestCommConfigPromise) return guestCommConfigPromise;

  guestCommConfigPromise = apiFetch('/api/v1/guest/comm/config')
    .then((res) => {
      guestCommConfigCache = res?.data || {};
      return guestCommConfigCache;
    })
    .finally(() => {
      guestCommConfigPromise = null;
    });

  return guestCommConfigPromise;
}

function isCaptchaEnabled(config) {
  return Number(config?.is_captcha || 0) === 1;
}

function getCaptchaProvider(config) {
  return String(config?.captcha_type || '').toLowerCase();
}

function isTurnstileCaptchaEnabled(config) {
  return isCaptchaEnabled(config) && getCaptchaProvider(config) === 'turnstile';
}

function isPowEnabled(config) {
  return Number(config?.pow_enable || 0) === 1;
}

function getPowProofByMode(mode) {
  return String(mode || '').toLowerCase() === 'register'
    ? loginCaptchaState.registerPowProof
    : loginCaptchaState.loginPowProof;
}

function setPowProofByMode(mode, proof) {
  if (String(mode || '').toLowerCase() === 'register') {
    loginCaptchaState.registerPowProof = proof || null;
  } else {
    loginCaptchaState.loginPowProof = proof || null;
  }
  updateAuthSubmitAvailability();
}

function consumePowProofByMode(mode) {
  const proof = getPowProofByMode(mode);
  setPowProofByMode(mode, null);
  ensurePowProofForMode(mode).catch(() => {});
  return proof;
}

function bufferToHex(buffer) {
  const bytes = new Uint8Array(buffer);
  let hex = '';
  for (let i = 0; i < bytes.length; i += 1) {
    hex += bytes[i].toString(16).padStart(2, '0');
  }
  return hex;
}

function sha256Fallback(message) {
  const utf8 = unescape(encodeURIComponent(String(message || '')));
  const constants = [
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
      const t1 = (hh + (s3 ^ s4 ^ s5) + ch + constants[j] + w[j]) | 0;

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

  return [a, b, c, d, e, f, g, h]
    .map((value) => (value >>> 0).toString(16).padStart(8, '0'))
    .join('');
}

async function sha256Hex(message) {
  if (window.crypto && window.crypto.subtle && window.TextEncoder) {
    const data = new TextEncoder().encode(String(message || ''));
    const digest = await window.crypto.subtle.digest('SHA-256', data);
    return bufferToHex(digest);
  }
  return sha256Fallback(message);
}

function yieldControl() {
  return new Promise((resolve) => {
    if (window.requestIdleCallback) {
      window.requestIdleCallback(() => resolve());
      return;
    }
    setTimeout(resolve, 0);
  });
}

async function loadPowChallenge() {
  const res = await fetch(POW_CHALLENGE_URL, { credentials: 'same-origin' });
  const text = await res.text();
  let json = null;
  try {
    json = text ? JSON.parse(text) : null;
  } catch (_) {
    json = null;
  }
  if (!res.ok) {
    throw new Error(json?.message || json?.error || `HTTP ${res.status}`);
  }
  return json?.data || null;
}

async function computePowProof(challenge) {
  if (!challenge || typeof challenge !== 'object') return null;
  const difficulty = Number(challenge.difficulty || 4);
  const prefix = '0'.repeat(Math.max(1, difficulty));
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
}

async function ensurePowProofForMode(mode = 'login') {
  const normalizedMode = String(mode || 'login').toLowerCase() === 'register' ? 'register' : 'login';
  const config = loginCaptchaState.config || await loadGuestCommConfig().catch(() => ({}));
  loginCaptchaState.config = config;

  if (!isPowEnabled(config)) {
    setPowProofByMode(normalizedMode, null);
    return null;
  }

  const current = getPowProofByMode(normalizedMode);
  if (current?.expires_at && Number(current.expires_at) > Math.floor(Date.now() / 1000) + 5) {
    return current;
  }

  if (loginCaptchaState.powPromises[normalizedMode]) {
    return loginCaptchaState.powPromises[normalizedMode];
  }

  loginCaptchaState.powPromises[normalizedMode] = (async () => {
    const challenge = await loadPowChallenge();
    const proof = await computePowProof(challenge);
    setPowProofByMode(normalizedMode, proof);
    return proof;
  })()
    .catch((error) => {
      setPowProofByMode(normalizedMode, null);
      throw error;
    })
    .finally(() => {
      loginCaptchaState.powPromises[normalizedMode] = null;
      updateAuthSubmitAvailability();
    });

  updateAuthSubmitAvailability();
  return loginCaptchaState.powPromises[normalizedMode];
}

function setPowHint(message = '', isError = false) {
  const hintEl = qs('#loginPowHint');
  if (!hintEl) return;

  const text = String(message || '').trim();
  hintEl.classList.toggle('hidden', !text);
  hintEl.classList.toggle('error', Boolean(isError));
  hintEl.textContent = text;
}

function updateAuthSubmitAvailability(activePanel = '') {
  const panel = String(activePanel || '').toLowerCase() === 'register'
    ? 'register'
    : (qs('#regBtn') ? 'register' : 'login');
  const submitBtn = qs(panel === 'register' ? '#regBtn' : '#loginBtn');
  if (!submitBtn) return;

  const config = loginCaptchaState.config || guestCommConfigCache || {};
  const provider = getCaptchaProvider(config);
  let captchaReady = true;
  if (isCaptchaEnabled(config)) {
    if (provider === 'turnstile' || provider === 'recaptcha') {
      captchaReady = Boolean(panel === 'register' ? loginCaptchaState.registerToken : loginCaptchaState.loginToken);
    } else if (provider === 'recaptcha-v3') {
      captchaReady = Boolean(loginCaptchaState.recaptchaV3Ready);
    } else {
      captchaReady = false;
    }
  }
  const powReady = !isPowEnabled(config) || Boolean(getPowProofByMode(panel));

  submitBtn.disabled = !(captchaReady && powReady);

  if (!isPowEnabled(config)) {
    setPowHint('');
    return;
  }

  if (powReady) {
    setPowHint(`防刷验证已准备完成，当前难度 ${Number(config?.pow_effective_difficulty || config?.pow_difficulty || 4)}。`);
    return;
  }

  if (loginCaptchaState.powPromises[panel]) {
    setPowHint(`正在准备防刷验证，当前难度 ${Number(config?.pow_effective_difficulty || config?.pow_difficulty || 4)}，请稍候...`);
    return;
  }

  setPowHint('防刷验证未就绪，请稍候重试。', true);
}

function ensureTurnstilePreconnect() {
  if (document.getElementById(TURNSTILE_PRECONNECT_ID)) return;
  const link = document.createElement('link');
  link.id = TURNSTILE_PRECONNECT_ID;
  link.rel = 'preconnect';
  link.href = 'https://challenges.cloudflare.com';
  link.crossOrigin = 'anonymous';
  document.head.appendChild(link);
}

function ensureRecaptchaPreconnect() {
  if (document.getElementById(RECAPTCHA_PRECONNECT_ID)) return;
  const link = document.createElement('link');
  link.id = RECAPTCHA_PRECONNECT_ID;
  link.rel = 'preconnect';
  link.href = 'https://www.recaptcha.net';
  link.crossOrigin = 'anonymous';
  document.head.appendChild(link);
}

function isRecaptchaReady(provider = 'recaptcha') {
  const p = String(provider || 'recaptcha').toLowerCase();
  if (!window.grecaptcha) return false;
  if (p === 'recaptcha-v3') {
    return typeof window.grecaptcha.ready === 'function' && typeof window.grecaptcha.execute === 'function';
  }
  return typeof window.grecaptcha.render === 'function';
}

function resolveRecaptchaScriptSrc(provider = 'recaptcha', siteKey = '') {
  const p = String(provider || 'recaptcha').toLowerCase();
  if (p === 'recaptcha-v3') {
    const key = String(siteKey || '').trim();
    return `https://www.recaptcha.net/recaptcha/api.js?render=${encodeURIComponent(key)}`;
  }
  return 'https://www.recaptcha.net/recaptcha/api.js?render=explicit';
}

async function ensureRecaptchaScriptLoaded(provider = 'recaptcha', siteKey = '') {
  const p = String(provider || 'recaptcha').toLowerCase();
  const desiredSrc = resolveRecaptchaScriptSrc(p, siteKey);
  if (!desiredSrc) throw new Error('reCAPTCHA 脚本地址不可用');

  if (loginCaptchaState.recaptchaScriptPromise && loginCaptchaState.recaptchaScriptSrc === desiredSrc) {
    return loginCaptchaState.recaptchaScriptPromise;
  }

  if (isRecaptchaReady(p) && loginCaptchaState.recaptchaScriptSrc === desiredSrc) {
    return;
  }

  const existed = document.getElementById(RECAPTCHA_SCRIPT_ID);
  const existedSrc = existed?.getAttribute('src') || '';
  if (existed && existedSrc !== desiredSrc) {
    existed.remove();
  }

  loginCaptchaState.recaptchaScriptSrc = desiredSrc;

  // If script already exists with same src but grecaptcha isn't ready yet, poll for readiness.
  if (document.getElementById(RECAPTCHA_SCRIPT_ID) && existedSrc === desiredSrc && !isRecaptchaReady(p)) {
    loginCaptchaState.recaptchaScriptPromise = new Promise((resolve, reject) => {
      let checks = 0;
      const timer = setInterval(() => {
        checks += 1;
        if (isRecaptchaReady(p)) {
          clearInterval(timer);
          resolve();
          return;
        }
        if (checks >= 80) {
          clearInterval(timer);
          reject(new Error('reCAPTCHA 脚本加载超时'));
        }
      }, 100);
    }).finally(() => {
      loginCaptchaState.recaptchaScriptPromise = null;
    });
    return loginCaptchaState.recaptchaScriptPromise;
  }

  ensureRecaptchaPreconnect();
  loginCaptchaState.recaptchaScriptPromise = new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.id = RECAPTCHA_SCRIPT_ID;
    script.src = desiredSrc;
    script.async = true;
    script.defer = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('reCAPTCHA 脚本加载失败'));
    document.head.appendChild(script);
  })
    .then(() => new Promise((resolve, reject) => {
      let checks = 0;
      const timer = setInterval(() => {
        checks += 1;
        if (isRecaptchaReady(p)) {
          clearInterval(timer);
          resolve();
          return;
        }
        if (checks >= 80) {
          clearInterval(timer);
          reject(new Error('reCAPTCHA 脚本初始化超时'));
        }
      }, 100);
    }))
    .finally(() => {
      loginCaptchaState.recaptchaScriptPromise = null;
    });

  return loginCaptchaState.recaptchaScriptPromise;
}

function resetRecaptchaWidget(widgetId) {
  if (widgetId === null || widgetId === undefined) return;
  try {
    if (window.grecaptcha && typeof window.grecaptcha.reset === 'function') {
      window.grecaptcha.reset(widgetId);
    }
  } catch (_) {}
}

async function executeRecaptchaV3(siteKey, action = 'portal_login') {
  const key = String(siteKey || '').trim();
  if (!key) throw new Error('reCAPTCHA v3 site key 未配置');
  await ensureRecaptchaScriptLoaded('recaptcha-v3', key);

  return new Promise((resolve, reject) => {
    try {
      window.grecaptcha.ready(() => {
        Promise.resolve(window.grecaptcha.execute(key, { action }))
          .then((token) => resolve(String(token || '').trim()))
          .catch(reject);
      });
    } catch (e) {
      reject(e);
    }
  });
}

function prewarmLoginCaptcha() {
  const cached = loginCaptchaState.config || guestCommConfigCache;
  if (cached) {
    loginCaptchaState.config = cached;
    if (isCaptchaEnabled(cached)) {
      const provider = getCaptchaProvider(cached);
      if (provider === 'turnstile') {
        ensureTurnstilePreconnect();
        ensureTurnstileScriptLoaded().catch(() => {});
      } else if (provider === 'recaptcha') {
        ensureRecaptchaScriptLoaded('recaptcha').catch(() => {});
      } else if (provider === 'recaptcha-v3') {
        ensureRecaptchaScriptLoaded('recaptcha-v3', cached?.recaptcha_v3_site_key || '').catch(() => {});
      }
    }
    return;
  }

  loadGuestCommConfig()
    .then((cfg) => {
      loginCaptchaState.config = cfg || {};
      if (isCaptchaEnabled(cfg)) {
        const provider = getCaptchaProvider(cfg);
        if (provider === 'turnstile') {
          ensureTurnstilePreconnect();
          return ensureTurnstileScriptLoaded();
        }
        if (provider === 'recaptcha') {
          return ensureRecaptchaScriptLoaded('recaptcha');
        }
        if (provider === 'recaptcha-v3') {
          return ensureRecaptchaScriptLoaded('recaptcha-v3', cfg?.recaptcha_v3_site_key || '');
        }
      }
      return null;
    })
    .catch(() => {});
}

async function ensureTurnstileScriptLoaded() {
  if (window.turnstile && typeof window.turnstile.render === 'function') {
    return;
  }

  if (loginCaptchaState.turnstileScriptPromise) {
    return loginCaptchaState.turnstileScriptPromise;
  }

  const existed = document.getElementById(TURNSTILE_SCRIPT_ID);
  if (existed) {
    loginCaptchaState.turnstileScriptPromise = new Promise((resolve, reject) => {
      let checks = 0;
      const timer = setInterval(() => {
        checks += 1;
        if (window.turnstile && typeof window.turnstile.render === 'function') {
          clearInterval(timer);
          resolve();
          return;
        }
        if (checks >= 50) {
          clearInterval(timer);
          reject(new Error('Turnstile 脚本加载超时'));
        }
      }, 100);
    }).finally(() => {
      loginCaptchaState.turnstileScriptPromise = null;
    });
    return loginCaptchaState.turnstileScriptPromise;
  }

  loginCaptchaState.turnstileScriptPromise = new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.id = TURNSTILE_SCRIPT_ID;
    script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';
    script.async = true;
    script.defer = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Turnstile 脚本加载失败'));
    document.head.appendChild(script);
  }).finally(() => {
    loginCaptchaState.turnstileScriptPromise = null;
  });

  return loginCaptchaState.turnstileScriptPromise;
}

function resetTurnstileWidget(widgetId) {
  if (widgetId === null || widgetId === undefined) return;
  try {
    if (window.turnstile && typeof window.turnstile.reset === 'function') {
      window.turnstile.reset(widgetId);
    }
  } catch (_) {}
}

async function setupLoginCaptchaUi(activePanel = 'login') {
  const mode = String(activePanel || '').toLowerCase() === 'register' ? 'register' : 'login';
  const captchaWrap = qs(mode === 'register' ? '#regTurnstileWrap' : '#loginTurnstileWrap');
  const hintEl = qs('#loginCaptchaHint');

  if (!captchaWrap || !hintEl) return;

  // 切换面板时，清理旧 token，避免跨面板复用
  if (mode === 'login') {
    loginCaptchaState.loginToken = '';
    loginCaptchaState.turnstileLoginWidgetId = null;
    loginCaptchaState.recaptchaLoginWidgetId = null;
    loginCaptchaState.registerToken = '';
    loginCaptchaState.turnstileRegisterWidgetId = null;
    loginCaptchaState.recaptchaRegisterWidgetId = null;
  } else {
    loginCaptchaState.registerToken = '';
    loginCaptchaState.turnstileRegisterWidgetId = null;
    loginCaptchaState.recaptchaRegisterWidgetId = null;
    loginCaptchaState.loginToken = '';
    loginCaptchaState.turnstileLoginWidgetId = null;
    loginCaptchaState.recaptchaLoginWidgetId = null;
  }
  loginCaptchaState.recaptchaV3Ready = false;

  const showHint = (text, isError = false) => {
    hintEl.classList.remove('hidden');
    hintEl.textContent = text;
    hintEl.classList.toggle('error', Boolean(isError));
  };

  captchaWrap.classList.add('hidden');
  hintEl.classList.remove('error');
  hintEl.classList.add('hidden');
  hintEl.textContent = '';
  updateAuthSubmitAvailability(mode);

  // 先做预热，尽量减少首屏等待时间
  prewarmLoginCaptcha();

  let config = loginCaptchaState.config || {};
  if (!config || typeof config !== 'object' || Object.keys(config).length === 0) {
    showHint('正在加载人机验证配置，请稍候...');
  }
  try {
    config = loginCaptchaState.config && Object.keys(loginCaptchaState.config).length
      ? loginCaptchaState.config
      : await loadGuestCommConfig();
    loginCaptchaState.config = config;
  } catch (_) {
    showHint('加载人机验证配置失败，请刷新后重试。', true);
    return;
  }

  if (!isCaptchaEnabled(config)) {
    return;
  }

  const provider = getCaptchaProvider(config);
  const widgetContainer = qs(mode === 'register' ? '#regTurnstileWidget' : '#loginTurnstileWidget');
  if (!widgetContainer) {
    showHint('验证码容器缺失，请刷新后重试。', true);
    return;
  }

  if (provider === 'turnstile') {
    const siteKey = String(config?.turnstile_site_key || '').trim();
    if (!siteKey) {
      showHint('Cloudflare Turnstile 的站点密钥未配置。', true);
      return;
    }

    showHint('正在加载人机验证控件，请稍候...');
    try {
      ensureTurnstilePreconnect();
      await ensureTurnstileScriptLoaded();
    } catch (e) {
      showHint(e.message || 'Turnstile 脚本加载失败', true);
      return;
    }

    if (!(window.turnstile && typeof window.turnstile.render === 'function')) {
      showHint('Turnstile 初始化失败，请刷新后重试。', true);
      return;
    }

    widgetContainer.innerHTML = '';
    const widgetId = window.turnstile.render(widgetContainer, {
      sitekey: siteKey,
      action: mode === 'register' ? 'portal_register' : 'portal_login',
      callback: (token) => {
        if (mode === 'register') {
          loginCaptchaState.registerToken = String(token || '');
        } else {
          loginCaptchaState.loginToken = String(token || '');
        }
        updateAuthSubmitAvailability(mode);
      },
      'expired-callback': () => {
        if (mode === 'register') {
          loginCaptchaState.registerToken = '';
        } else {
          loginCaptchaState.loginToken = '';
        }
        updateAuthSubmitAvailability(mode);
      },
      'error-callback': () => {
        if (mode === 'register') {
          loginCaptchaState.registerToken = '';
        } else {
          loginCaptchaState.loginToken = '';
        }
        updateAuthSubmitAvailability(mode);
      },
    });

    captchaWrap.classList.remove('hidden');
    showHint(mode === 'register' ? '请先完成人机验证，再进行注册。' : '请先完成人机验证，再进行登录。');

    if (mode === 'register') {
      loginCaptchaState.turnstileRegisterWidgetId = widgetId;
    } else {
      loginCaptchaState.turnstileLoginWidgetId = widgetId;
    }
    updateAuthSubmitAvailability(mode);
    return;
  }

  if (provider === 'recaptcha') {
    const siteKey = String(config?.recaptcha_site_key || '').trim();
    if (!siteKey) {
      showHint('Google reCAPTCHA 的站点密钥未配置。', true);
      return;
    }

    showHint('正在加载 reCAPTCHA 控件，请稍候...');
    try {
      await ensureRecaptchaScriptLoaded('recaptcha');
    } catch (e) {
      showHint(e.message || 'reCAPTCHA 脚本加载失败', true);
      return;
    }

    if (!(window.grecaptcha && typeof window.grecaptcha.render === 'function')) {
      showHint('reCAPTCHA 初始化失败，请刷新后重试。', true);
      return;
    }

    widgetContainer.innerHTML = '';
    const widgetId = window.grecaptcha.render(widgetContainer, {
      sitekey: siteKey,
      callback: (token) => {
        if (mode === 'register') {
          loginCaptchaState.registerToken = String(token || '');
        } else {
          loginCaptchaState.loginToken = String(token || '');
        }
        updateAuthSubmitAvailability(mode);
      },
      'expired-callback': () => {
        if (mode === 'register') {
          loginCaptchaState.registerToken = '';
        } else {
          loginCaptchaState.loginToken = '';
        }
        updateAuthSubmitAvailability(mode);
      },
      'error-callback': () => {
        if (mode === 'register') {
          loginCaptchaState.registerToken = '';
        } else {
          loginCaptchaState.loginToken = '';
        }
        updateAuthSubmitAvailability(mode);
      },
    });

    captchaWrap.classList.remove('hidden');
    showHint(mode === 'register' ? '请先完成人机验证，再进行注册。' : '请先完成人机验证，再进行登录。');

    if (mode === 'register') {
      loginCaptchaState.recaptchaRegisterWidgetId = widgetId;
    } else {
      loginCaptchaState.recaptchaLoginWidgetId = widgetId;
    }
    updateAuthSubmitAvailability(mode);
    return;
  }

  if (provider === 'recaptcha-v3') {
    const siteKey = String(config?.recaptcha_v3_site_key || '').trim();
    if (!siteKey) {
      showHint('reCAPTCHA v3 的站点密钥未配置。', true);
      loginCaptchaState.recaptchaV3Ready = false;
      updateAuthSubmitAvailability(mode);
      return;
    }

    showHint('当前启用 reCAPTCHA v3，无需手动验证，将在提交时自动完成。');
    try {
      await ensureRecaptchaScriptLoaded('recaptcha-v3', siteKey);
      loginCaptchaState.recaptchaV3Ready = true;
    } catch (e) {
      loginCaptchaState.recaptchaV3Ready = false;
      showHint(e.message || 'reCAPTCHA v3 脚本加载失败', true);
      return;
    } finally {
      updateAuthSubmitAvailability(mode);
    }
    return;
  }

  showHint('当前开启的人机验证类型暂未在此主题实现，请在后台切换为 Turnstile 或 reCAPTCHA。', true);
}

async function buildCaptchaPayloadForMode(mode, config) {
  const normalizedMode = String(mode || '').toLowerCase() === 'register' ? 'register' : 'login';
  if (!isCaptchaEnabled(config)) return {};

  const provider = getCaptchaProvider(config);
  const token = String(normalizedMode === 'register' ? loginCaptchaState.registerToken : loginCaptchaState.loginToken);

  if (provider === 'turnstile') {
    if (!token) {
      throw new Error('请先完成人机验证');
    }
    return { turnstile_token: token };
  }

  if (provider === 'recaptcha') {
    if (!token) {
      throw new Error('请先完成人机验证');
    }
    return { recaptcha_data: token };
  }

  if (provider === 'recaptcha-v3') {
    const siteKey = String(config?.recaptcha_v3_site_key || '').trim();
    if (!siteKey) {
      throw new Error('reCAPTCHA v3 的站点密钥未配置');
    }
    const action = normalizedMode === 'register' ? 'portal_register' : 'portal_login';
    const v3Token = await executeRecaptchaV3(siteKey, action);
    if (!v3Token) {
      throw new Error('reCAPTCHA v3 执行失败，请刷新后重试');
    }
    return { recaptcha_v3_token: v3Token };
  }

  throw new Error('当前开启的人机验证类型暂未在此主题实现，请在后台切换为 Turnstile 或 reCAPTCHA。');
}

function resetCaptchaUiForMode(mode, config) {
  const normalizedMode = String(mode || '').toLowerCase() === 'register' ? 'register' : 'login';
  if (!isCaptchaEnabled(config)) return;

  const provider = getCaptchaProvider(config);
  if (normalizedMode === 'register') {
    loginCaptchaState.registerToken = '';
  } else {
    loginCaptchaState.loginToken = '';
  }

  if (provider === 'turnstile') {
    resetTurnstileWidget(
      normalizedMode === 'register'
        ? loginCaptchaState.turnstileRegisterWidgetId
        : loginCaptchaState.turnstileLoginWidgetId
    );
  } else if (provider === 'recaptcha') {
    resetRecaptchaWidget(
      normalizedMode === 'register'
        ? loginCaptchaState.recaptchaRegisterWidgetId
        : loginCaptchaState.recaptchaLoginWidgetId
    );
  }

  updateAuthSubmitAvailability(normalizedMode);
}

function requireAuth() {
  if (!store.auth) {
    syncAuthNav();
    setPostLoginRedirect(location.hash || '#/dashboard');
    location.hash = '#/login';
    return false;
  }
  return true;
}

function formatTs(ts) {
  if (!ts) return '-';
  const d = new Date(ts * 1000);
  return isNaN(d.getTime()) ? String(ts) : d.toLocaleString();
}

function formatTrafficKb(kb) {
  let value = Number(kb) || 0;
  if (!Number.isFinite(value) || value < 0) value = 0;
  const gb = value / 1024 / 1024;
  const digits = gb >= 100 ? 0 : (gb >= 10 ? 1 : 2);
  return `${gb.toFixed(digits)} GB`;
}

function formatTrafficDisplay(kb, isUnlimited = false) {
  return isUnlimited ? '无限' : formatTrafficKb(kb);
}

function formatCompactNumber(value) {
  const num = Number(value);
  if (!Number.isFinite(num)) return '0';
  if (typeof Intl !== 'undefined' && Intl.NumberFormat) {
    return new Intl.NumberFormat('zh-CN', { notation: 'compact', maximumFractionDigits: 1 }).format(num);
  }
  return String(Math.round(num));
}

function formatMoneyCent(value) {
  const amount = Number(value);
  if (!Number.isFinite(amount)) return '¥0';
  const yuan = amount / 100;
  return `¥${yuan.toLocaleString(undefined, {
    minimumFractionDigits: Math.abs(amount % 100) > 0 ? 2 : 0,
    maximumFractionDigits: 2,
  })}`;
}

function formatRatePerSecond(value) {
  let num = Number(value);
  if (!Number.isFinite(num) || num <= 0) return '0 B/s';
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  let idx = 0;
  while (num >= 1024 && idx < units.length - 1) {
    num /= 1024;
    idx += 1;
  }
  const digits = num >= 100 ? 0 : (num >= 10 ? 1 : 2);
  return `${num.toFixed(digits)} ${units[idx]}`;
}

function formatDurationShort(seconds) {
  const total = Math.max(0, Math.floor(Number(seconds) || 0));
  if (total < 60) return `${total}s`;
  if (total < 3600) return `${Math.floor(total / 60)}m ${total % 60}s`;
  if (total < 86400) return `${Math.floor(total / 3600)}h ${Math.floor((total % 3600) / 60)}m`;
  return `${Math.floor(total / 86400)}d ${Math.floor((total % 86400) / 3600)}h`;
}

function formatPercent(value) {
  const num = Number(value);
  if (!Number.isFinite(num)) return '-';
  if (Math.abs(num) >= 100) return `${Math.round(num)}%`;
  if (Math.abs(num) >= 10) return `${num.toFixed(1)}%`;
  return `${num.toFixed(2)}%`;
}

function normalizeFreeQuotaMap(raw) {
  const source = raw && typeof raw === 'object' ? raw : {};
  const result = {};
  TRUST_LEVEL_OPTIONS.forEach((level) => {
    const value = Number(source[level] ?? source[String(level)] ?? 0);
    result[level] = Number.isFinite(value) && value > 0 ? Number(value.toFixed(2)) : 0;
  });
  return result;
}

function readFreeQuotaInputs(prefix, root = document) {
  const data = {};
  let hasPositive = false;
  TRUST_LEVEL_OPTIONS.forEach((level) => {
    const raw = String(qs(`#${prefix}${level}`, root)?.value || '').trim();
    const value = Number(raw || '0');
    const normalized = Number.isFinite(value) && value > 0 ? Number(value.toFixed(2)) : 0;
    data[level] = normalized;
    if (normalized > 0) hasPositive = true;
  });
  return hasPositive ? data : null;
}

function renderFreeQuotaInputFields(prefix, values) {
  return TRUST_LEVEL_OPTIONS.map((level) => {
    const val = Number(values?.[level] ?? values?.[String(level)] ?? 0);
    const text = val > 0 ? String(val) : '';
    return `
      <div class="field">
        <label>等级 ${level} 每月免费流量(GB)</label>
        <input id="${prefix}${level}" type="number" min="0" step="0.01" value="${text}" placeholder="0 表示不提供">
      </div>
    `;
  }).join('');
}

function serializeNodeLocationOption(option) {
  if (!option) return '';
  return `${option.code} | ${option.name}`;
}

function getNodeLocationInputValue(locationCode = '', locationName = '') {
  const code = String(locationCode || '').trim().toUpperCase();
  const name = String(locationName || '').trim();
  if (!code && !name) return '';
  const matched = NODE_LOCATION_OPTIONS.find((item) => item.code === code);
  if (matched) return serializeNodeLocationOption(matched);
  return code ? `${code} | ${name || code}` : name;
}

function renderNodeLocationDatalist() {
  return html`
    <datalist id="nodeLocationOptions">
      ${NODE_LOCATION_OPTIONS.map((item) => `<option value="${escapeHtml(serializeNodeLocationOption(item))}"></option>`).join('')}
    </datalist>
  `;
}

function parseNodeLocationSelection(rawValue) {
  const raw = String(rawValue || '').trim();
  if (!raw) return null;

  const normalized = raw.toLowerCase();
  const exact = NODE_LOCATION_OPTIONS.find((item) => {
    const serialized = serializeNodeLocationOption(item).toLowerCase();
    return serialized === normalized
      || item.code.toLowerCase() === normalized
      || item.name.toLowerCase() === normalized;
  });

  if (exact) {
    return { location_code: exact.code, location_name: exact.name };
  }

  const parts = raw.split('|').map((item) => item.trim()).filter(Boolean);
  if (parts.length >= 2) {
    return {
      location_code: String(parts[0] || '').toUpperCase(),
      location_name: parts.slice(1).join(' | '),
    };
  }

  return null;
}

function readNodeLocationSelection(inputId) {
  const rawValue = qs(`#${inputId}`)?.value || '';
  const parsed = parseNodeLocationSelection(rawValue);
  if (!parsed?.location_code || !parsed?.location_name) {
    throw new Error('请选择有效的节点落地地区');
  }
  return parsed;
}

function renderTcpingAgentLocationDatalist() {
  return html`
    <datalist id="tcpingAgentLocationOptions">
      ${TCPING_AGENT_LOCATION_OPTIONS.map((item) => `<option value="${escapeHtml(serializeNodeLocationOption(item))}"></option>`).join('')}
    </datalist>
  `;
}

function renderCnProvinceDatalist() {
  return html`
    <datalist id="cnProvinceOptions">
      ${CN_PROVINCE_OPTIONS.map((name) => `<option value="${escapeHtml(name)}"></option>`).join('')}
    </datalist>
  `;
}

function parseTcpingAgentLocationSelection(rawValue) {
  const raw = String(rawValue || '').trim();
  if (!raw) return null;

  const normalized = raw.toLowerCase();
  const exact = TCPING_AGENT_LOCATION_OPTIONS.find((item) => {
    const serialized = serializeNodeLocationOption(item).toLowerCase();
    return serialized === normalized
      || item.code.toLowerCase() === normalized
      || item.name.toLowerCase() === normalized;
  });

  if (exact) {
    return { location_code: exact.code, location_name: exact.name };
  }

  const parts = raw.split('|').map((item) => item.trim()).filter(Boolean);
  if (parts.length >= 2) {
    return {
      location_code: String(parts[0] || '').toUpperCase(),
      location_name: parts.slice(1).join(' | '),
    };
  }

  return null;
}

function readTcpingAgentLocationSelection(inputId) {
  const rawValue = qs(`#${inputId}`)?.value || '';
  const parsed = parseTcpingAgentLocationSelection(rawValue);
  if (!parsed?.location_code || !parsed?.location_name) {
    throw new Error('请选择有效的 TCPing Agent 归属地区');
  }
  return parsed;
}

function formatLatencyMs(value) {
  if (value === null || value === undefined || value === '') return '-';
  const latency = Number(value);
  if (!Number.isFinite(latency) || latency < 0) return '-';
  return `${Math.round(latency)} ms`;
}

function renderTcpingStatusPill(status, enabled = true) {
  const key = String(status || 'unknown').toLowerCase();
  if (key === 'unsupported') return '<span class="pill">不支持 UDP 探测</span>';
  if (!enabled) return '<span class="pill">未开启</span>';
  if (key === 'online') return '<span class="pill ok">在线</span>';
  if (key === 'offline') return '<span class="pill bad">离线</span>';
  return '<span class="pill warn">待探测</span>';
}

function renderNodeTrafficCell(node = {}) {
  const limit = Number(node?.traffic_limit || 0);
  const used = Number(node?.traffic_used || 0);
  const percent = Number(node?.traffic_usage_percentage || 0);
  if (limit > 0) {
    return html`
      <div class="muted">${escapeHtml(formatTrafficKb(used))} / ${escapeHtml(formatTrafficKb(limit))}</div>
      ${renderUsageBar(percent)}
    `;
  }

  return html`
    <div class="muted">${escapeHtml(formatTrafficKb(used))} / 无限</div>
    <div class="muted" style="margin-top:4px;">未设置节点总流量上限</div>
  `;
}

function renderNodeTcpingCell(node = {}) {
  const statusKey = String(node?.tcping_status || 'unknown').toLowerCase();
  if (statusKey === 'unsupported') {
    return html`
      <div>${renderTcpingStatusPill('unsupported', false)}</div>
      <div class="muted" style="margin-top:4px;">TCPing 不支持 UDP 探测</div>
      <div class="muted">未配置 TCPing 探测端口（TCP）</div>
    `;
  }

  return html`
    <div>${renderTcpingStatusPill(node?.tcping_status, Boolean(node?.tcping_enabled))}</div>
    <div class="muted" style="margin-top:4px;">
      ${node?.tcping_last_latency_ms !== null && node?.tcping_last_latency_ms !== undefined
        ? `最近 ${escapeHtml(formatLatencyMs(node.tcping_last_latency_ms))}`
        : '暂无延迟数据'}
    </div>
    <div class="muted">
      ${node?.tcping_last_sampled_at ? escapeHtml(formatTs(node.tcping_last_sampled_at)) : '未采样'}
    </div>
  `;
}

function maskToken(token) {
  const text = String(token || '');
  if (text.length <= 10) return text || '-';
  return `${text.slice(0, 6)}...${text.slice(-4)}`;
}

function buildTcpingChart(samples, width = 960, height = 240) {
  const rows = Array.isArray(samples) ? samples : [];
  if (!rows.length) {
    return '<div class="notice">当前时间范围内还没有 TCPing 采样。</div>';
  }

  const chartHeight = height - 50;
  const chartWidth = width - 56;
  const reachable = rows.filter((item) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)));
  const maxLatency = Math.max(50, ...reachable.map((item) => Number(item.latency_ms || 0)));
  const minTime = Number(rows[0]?.sampled_at || 0);
  const maxTime = Number(rows[rows.length - 1]?.sampled_at || minTime + 1);
  const span = Math.max(1, maxTime - minTime);

  const xFor = (ts) => 28 + (((Number(ts || 0) - minTime) / span) * chartWidth);
  const yFor = (latency) => 18 + (chartHeight - ((Number(latency || 0) / maxLatency) * chartHeight));

  const reachablePoints = rows
    .filter((item) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)))
    .map((item) => `${xFor(item.sampled_at).toFixed(2)},${yFor(item.latency_ms).toFixed(2)}`)
    .join(' ');

  const offlineDots = rows
    .filter((item) => !item?.is_reachable)
    .map((item) => ({
      x: xFor(item.sampled_at),
      y: height - 24,
    }));

  const yTicks = [0.25, 0.5, 0.75, 1].map((ratio) => {
    const latency = Math.round(maxLatency * ratio);
    const y = yFor(latency);
    return {
      latency,
      y,
    };
  });

  return html`
    <div class="chart-shell">
      <svg viewBox="0 0 ${width} ${height}" class="tcping-chart" role="img" aria-label="TCPing latency chart">
        <rect x="0" y="0" width="${width}" height="${height}" rx="18" fill="rgba(255,255,255,0.66)"></rect>
        ${yTicks.map((tick) => `
          <g>
            <line x1="28" y1="${tick.y}" x2="${width - 20}" y2="${tick.y}" stroke="rgba(15,23,42,0.10)" stroke-dasharray="4 6"></line>
            <text x="6" y="${tick.y + 4}" font-size="11" fill="rgba(71,85,105,0.92)">${tick.latency}ms</text>
          </g>
        `).join('')}
        <line x1="28" y1="18" x2="28" y2="${height - 24}" stroke="rgba(15,23,42,0.16)"></line>
        <line x1="28" y1="${height - 24}" x2="${width - 20}" y2="${height - 24}" stroke="rgba(15,23,42,0.16)"></line>
        ${reachablePoints
          ? `<polyline fill="none" stroke="rgba(15,118,110,0.95)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" points="${reachablePoints}"></polyline>`
          : ''}
        ${offlineDots.map((dot) => `<circle cx="${dot.x.toFixed(2)}" cy="${dot.y.toFixed(2)}" r="4" fill="rgba(220,38,38,0.92)"></circle>`).join('')}
      </svg>
      <div class="chart-legend">
        <span><span class="chart-dot ok"></span> 可连通延迟</span>
        <span><span class="chart-dot bad"></span> 不可连通</span>
      </div>
    </div>
  `;
}

const TCPING_SERIES_COLORS = Object.freeze([
  'rgba(15,118,110,0.95)', // teal
  'rgba(37,99,235,0.92)', // blue
  'rgba(79,70,229,0.92)', // indigo
  'rgba(124,58,237,0.92)', // violet
  'rgba(217,119,6,0.92)', // amber
  'rgba(16,185,129,0.92)', // emerald
  'rgba(14,116,144,0.92)', // cyan
  'rgba(234,88,12,0.92)', // orange
  'rgba(71,85,105,0.92)', // slate
  'rgba(225,29,72,0.92)', // rose
]);

function hashString32(input) {
  const text = String(input || '');
  let h = 2166136261;
  for (let i = 0; i < text.length; i += 1) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function pickTcpingSeriesColor(key) {
  const idx = hashString32(key) % TCPING_SERIES_COLORS.length;
  return TCPING_SERIES_COLORS[idx];
}

function buildTcpingSeriesByRegion(samples, agents = []) {
  const rows = Array.isArray(samples) ? samples : [];
  const agentRows = Array.isArray(agents) ? agents : [];
  const agentMap = Object.create(null);
  agentRows.forEach((agent) => {
    const id = Number(agent?.id || 0);
    if (Number.isFinite(id) && id > 0) {
      agentMap[id] = agent;
    }
  });

  const buckets = new Map();
  rows.forEach((item) => {
    const agentId = item?.agent_id !== null && item?.agent_id !== undefined ? Number(item.agent_id) : 0;
    const agent = agentId > 0 ? agentMap[agentId] : null;
    const label = String(agent?.location_display || agent?.location_name || '未知地区').trim() || '未知地区';
    if (!buckets.has(label)) {
      buckets.set(label, []);
    }
    buckets.get(label).push(item);
  });

  const series = Array.from(buckets.entries())
    .map(([label, list]) => {
      const hash = hashString32(label);
      return {
        key: `r_${hash.toString(16)}_${label.length}`,
        label,
        color: pickTcpingSeriesColor(label),
        samples: Array.isArray(list) ? list : [],
      };
    })
    .sort((a, b) => a.label.localeCompare(b.label, 'zh-CN'));

  return series;
}

function buildTcpingChartBySeries(seriesList, selectedKeys = null, width = 960, height = 240) {
  const list = Array.isArray(seriesList) ? seriesList : [];
  if (!list.length) return '<div class="notice">当前时间范围内还没有 TCPing 采样。</div>';

  const selectedSet = new Set(
    Array.isArray(selectedKeys) && selectedKeys.length ? selectedKeys.map((k) => String(k)) : list.map((s) => String(s.key))
  );
  const selected = list.filter((s) => selectedSet.has(String(s.key)));

  const allRows = [];
  selected.forEach((s) => {
    if (Array.isArray(s?.samples) && s.samples.length) {
      allRows.push(...s.samples);
    }
  });

  if (!allRows.length) {
    return '<div class="notice">当前筛选范围内还没有 TCPing 采样。</div>';
  }

  const chartHeight = height - 50;
  const chartWidth = width - 56;

  const reachable = allRows.filter((item) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)));
  const maxLatency = Math.max(50, ...reachable.map((item) => Number(item.latency_ms || 0)));

  let minTime = Infinity;
  let maxTime = 0;
  selected.forEach((s) => {
    const rows = Array.isArray(s?.samples) ? s.samples : [];
    if (!rows.length) return;
    const firstTs = Number(rows[0]?.sampled_at || 0);
    const lastTs = Number(rows[rows.length - 1]?.sampled_at || 0);
    if (Number.isFinite(firstTs) && firstTs > 0) minTime = Math.min(minTime, firstTs);
    if (Number.isFinite(lastTs) && lastTs > 0) maxTime = Math.max(maxTime, lastTs);
  });

  if (!Number.isFinite(minTime) || minTime <= 0) {
    return '<div class="notice">当前时间范围内还没有 TCPing 采样。</div>';
  }
  if (maxTime <= minTime) maxTime = minTime + 1;
  const span = Math.max(1, maxTime - minTime);

  const xFor = (ts) => 28 + (((Number(ts || 0) - minTime) / span) * chartWidth);
  const yFor = (latency) => 18 + (chartHeight - ((Number(latency || 0) / maxLatency) * chartHeight));

  const yTicks = [0.25, 0.5, 0.75, 1].map((ratio) => {
    const latency = Math.round(maxLatency * ratio);
    const y = yFor(latency);
    return { latency, y };
  });

  const polylines = selected.map((s) => {
    const points = (Array.isArray(s.samples) ? s.samples : [])
      .filter((item) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)))
      .map((item) => `${xFor(item.sampled_at).toFixed(2)},${yFor(item.latency_ms).toFixed(2)}`)
      .join(' ');
    if (!points) return '';
    return `<polyline fill="none" stroke="${escapeHtml(String(s.color || 'rgba(15,118,110,0.95)'))}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" points="${points}"></polyline>`;
  }).join('');

  const offlineDots = selected.map((s, idx) => {
    const offset = Math.min(18, idx * 6);
    const y = height - 24 - offset;
    return (Array.isArray(s.samples) ? s.samples : [])
      .filter((item) => !item?.is_reachable)
      .map((item) => {
        const x = xFor(item.sampled_at);
        const stroke = escapeHtml(String(s.color || 'rgba(15,118,110,0.95)'));
        return `<circle cx="${x.toFixed(2)}" cy="${y.toFixed(2)}" r="4" fill="rgba(220,38,38,0.86)" stroke="${stroke}" stroke-width="2"></circle>`;
      }).join('');
  }).join('');

  return html`
    <div class="chart-shell">
      <svg viewBox="0 0 ${width} ${height}" class="tcping-chart" role="img" aria-label="TCPing latency chart (multi region)">
        <rect x="0" y="0" width="${width}" height="${height}" rx="18" fill="rgba(255,255,255,0.66)"></rect>
        ${yTicks.map((tick) => `
          <g>
            <line x1="28" y1="${tick.y}" x2="${width - 20}" y2="${tick.y}" stroke="rgba(15,23,42,0.10)" stroke-dasharray="4 6"></line>
            <text x="6" y="${tick.y + 4}" font-size="11" fill="rgba(71,85,105,0.92)">${tick.latency}ms</text>
          </g>
        `).join('')}
        <line x1="28" y1="18" x2="28" y2="${height - 24}" stroke="rgba(15,23,42,0.16)"></line>
        <line x1="28" y1="${height - 24}" x2="${width - 20}" y2="${height - 24}" stroke="rgba(15,23,42,0.16)"></line>
        ${polylines}
        ${offlineDots}
      </svg>
      <div class="chart-legend">
        <span><span class="chart-dot ok"></span> 可达延迟（颜色=探针地区）</span>
        <span><span class="chart-dot bad"></span> 不可连通（红底点，描边=探针地区）</span>
      </div>
    </div>
  `;
}

function renderUsageBar(percentRaw) {
  const percent = Math.max(0, Math.min(100, Number(percentRaw) || 0));
  const statusClass = percent >= 90 ? 'danger' : (percent >= 70 ? 'warn' : 'ok');
  return html`
    <div class="quota-meter ${statusClass}">
      <div class="quota-meter-fill" style="width:${percent.toFixed(2)}%"></div>
    </div>
    <div class="muted" style="margin-top:4px;">${percent.toFixed(2)}%</div>
  `;
}

function renderPlanQuotaTable(quotaData) {
  const items = Array.isArray(quotaData?.items) ? quotaData.items : [];
  if (!items.length) {
    return `<div class="muted">暂无生效中的套餐额度。</div>`;
  }

  return html`
    <table class="table" style="margin-top:10px;">
      <thead>
        <tr><th>套餐</th><th>周期</th><th>适用节点</th><th>有效期</th><th>已用 / 总量</th><th>剩余</th><th>进度</th></tr>
      </thead>
      <tbody>
        ${items.map((item) => {
          const isUnlimited = Boolean(item?.is_unlimited_traffic);
          const usageView = isUnlimited
            ? '<span class="muted">无限</span>'
            : renderUsageBar(item.usage_percent || 0);

          return html`
            <tr>
              <td>${escapeHtml(item.plan_name || ('套餐 #' + item.plan_id))}</td>
              <td class="muted">${escapeHtml(item.period || '-')}</td>
              <td class="muted">${Array.isArray(item.node_ids) && item.node_ids.length ? item.node_ids.join(', ') : '全部节点'}</td>
              <td class="muted">${item.expired_at ? formatTs(item.expired_at) : '长期有效'}</td>
              <td class="muted">${formatTrafficKb(item.used_traffic_kb)} / ${formatTrafficDisplay(item.traffic_allowance_kb, isUnlimited)}</td>
              <td class="muted">${formatTrafficDisplay(item.remaining_traffic_kb, isUnlimited)}</td>
              <td>${usageView}</td>
            </tr>
          `;
        }).join('')}
      </tbody>
    </table>
  `;
}

async function copyText(value) {
  const text = String(value || '').trim();
  if (!text) throw new Error('没有可复制的内容');

  if (navigator.clipboard && navigator.clipboard.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const input = document.createElement('textarea');
  input.value = text;
  input.setAttribute('readonly', 'readonly');
  input.style.position = 'fixed';
  input.style.left = '-9999px';
  document.body.appendChild(input);
  input.select();
  const ok = document.execCommand('copy');
  document.body.removeChild(input);
  if (!ok) throw new Error('复制失败，请手动复制');
}

function normalizeHttpUrl(raw) {
  const text = String(raw || '').trim();
  if (!text) return '';
  try {
    const parsed = new URL(text, location.origin);
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return '';
    return parsed.toString();
  } catch (_) {
    return '';
  }
}

function buildSafeEpayPostForm(raw) {
  const parsed = new DOMParser().parseFromString(String(raw || ''), 'text/html');
  const sourceForm = parsed.querySelector('form');
  if (!sourceForm) return null;

  const action = normalizeHttpUrl(sourceForm.getAttribute('action') || '');
  const method = String(sourceForm.getAttribute('method') || '').trim().toLowerCase();
  if (!action || method !== 'post') return null;

  const form = document.createElement('form');
  form.method = 'post';
  form.action = action;
  form.hidden = true;
  sourceForm.querySelectorAll('input[type="hidden"]').forEach((sourceInput) => {
    const name = String(sourceInput.getAttribute('name') || '').trim();
    if (!name || name.length > 256) return;
    const input = document.createElement('input');
    input.type = 'hidden';
    input.name = name;
    input.value = sourceInput.getAttribute('value') || '';
    form.append(input);
  });
  return form;
}

function submitSafeEpayPostForm(container, raw, message) {
  const form = buildSafeEpayPostForm(raw);
  if (!container || !form) throw new Error('支付网关返回了无效表单');

  const notice = document.createElement('div');
  notice.className = 'notice';
  notice.textContent = message;
  container.replaceChildren(notice, form);
  HTMLFormElement.prototype.submit.call(form);
}

function normalizePlanVisibilityScope(scope) {
  const raw = String(scope || '').trim().toLowerCase();
  if (raw === 'link_only') return 'link_only';
  if (raw === 'assigned_only') return 'assigned_only';
  return 'public';
}

function parseIdListInput(raw) {
  return String(raw || '')
    .split(/[\n,，]+/)
    .map((item) => item.trim())
    .filter(Boolean)
    .map((item) => Number(item))
    .filter((value) => Number.isFinite(value) && value > 0);
}

function buildNodePlanShareLink(token) {
  const text = String(token || '').trim();
  if (!text) return '';
  return `${location.origin}/app/#/plan-link/${encodeURIComponent(text)}`;
}

function hasPositiveTrialQuota(raw) {
  if (!raw || typeof raw !== 'object') return false;
  return Object.values(raw).some((value) => Number(value || 0) > 0);
}

function showDeployInfoModal({ nodeId, nodeName, panelUrl, command }) {
  const existed = document.getElementById('deployInfoModal');
  if (existed) existed.remove();

  const modal = document.createElement('div');
  modal.id = 'deployInfoModal';
  modal.className = 'modal-overlay';
  modal.innerHTML = html`
    <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="deployInfoTitle">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h3 id="deployInfoTitle">节点部署信息（#${escapeHtml(nodeId)} ${escapeHtml(nodeName || '')}）</h3>
        <button class="btn small" id="deployModalCloseTop">关闭</button>
      </div>
      <div class="grid" style="margin-top:10px;">
        <div class="field">
          <label>本站链接</label>
          <div class="row">
            <input id="deployPanelUrl" readonly value="${escapeHtml(panelUrl || location.origin)}">
            <button class="btn small" id="copyDeployPanelUrl">复制</button>
          </div>
        </div>
        <div class="field">
          <label>一次性部署命令</label>
          <textarea id="deployCommand" rows="4" readonly>${escapeHtml(command || '')}</textarea>
          <div class="row end"><button class="btn small" id="copyDeployCommand">复制命令</button></div>
        </div>
      </div>
      <div class="row end" style="margin-top:10px;">
        <button class="btn primary" id="deployModalCloseBottom">我知道了</button>
      </div>
    </div>
  `;

  document.body.appendChild(modal);
  const close = () => modal.remove();

  modal.addEventListener('click', (e) => {
    if (e.target === modal) close();
  });
  const closeTop = qs('#deployModalCloseTop', modal);
  const closeBottom = qs('#deployModalCloseBottom', modal);
  if (closeTop) closeTop.addEventListener('click', close);
  if (closeBottom) closeBottom.addEventListener('click', close);

  const copyBind = (btnId, inputId, okText) => {
    const btn = qs(`#${btnId}`, modal);
    const input = qs(`#${inputId}`, modal);
    if (!btn || !input) return;
    btn.addEventListener('click', async () => {
      try {
        await copyText(input.value);
        alert(okText);
      } catch (e) {
        alert(e.message || '复制失败');
      }
    });
  };

  copyBind('copyDeployPanelUrl', 'deployPanelUrl', '本站链接已复制');
  copyBind('copyDeployCommand', 'deployCommand', '部署命令已复制');
}

function showAdminNodePlanFormModal({ mode = 'new', plan = null, nodeOptions = [], onSubmit }) {
  const existed = document.getElementById('adminNodePlanModal');
  if (existed) existed.remove();

  const initialPrices = {};
  PLAN_PRICE_FIELDS.forEach(({ key }) => {
    if (plan && plan[key] !== null && plan[key] !== undefined) {
      initialPrices[key] = Number(plan[key]) || 0;
    }
  });
  const initialUnlimitedTraffic = Boolean(plan?.is_unlimited_traffic);
  const initialFreeQuota = normalizeFreeQuotaMap(plan?.free_quota_gb_by_trust_level);
  const initialVisibility = normalizePlanVisibilityScope(plan?.visibility_scope);
  const initialAccessUserIds = asArray(plan?.access_user_ids).join(',');
  const initialShareToken = String(plan?.share_token || '');
  const initialAllowTrial = Boolean(plan?.allow_trial || hasPositiveTrialQuota(plan?.free_quota_gb_by_trust_level));
  const selectedNodeIdSet = new Set(
    (Array.isArray(plan?.node_ids) ? plan.node_ids : [])
      .map((value) => Number(value))
      .filter((value) => Number.isFinite(value) && value > 0)
  );
  let normalizedNodeOptions = Array.isArray(nodeOptions)
    ? nodeOptions
      .map((node) => ({
        id: Number(node?.id || 0),
        name: String(node?.name || ''),
        owner_email: String(node?.owner_email || ''),
        user_id: Number(node?.user_id || 0),
        status: String(node?.status || 'unknown'),
        online_status: String(node?.online_status || ''),
        is_online: node?.is_online,
      }))
      .filter((node) => Number.isFinite(node.id) && node.id > 0)
    : [];
  if (!normalizedNodeOptions.length && selectedNodeIdSet.size) {
    normalizedNodeOptions = Array.from(selectedNodeIdSet).map((id) => ({
      id,
      name: `节点 #${id}`,
      owner_email: '',
      user_id: 0,
      status: 'unknown',
      online_status: '',
      is_online: null,
    }));
  }

  const modal = document.createElement('div');
  modal.id = 'adminNodePlanModal';
  modal.className = 'modal-overlay';
  modal.innerHTML = html`
    <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="admNodePlanTitle" style="max-width: 980px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h3 id="admNodePlanTitle">${mode === 'new' ? '新建节点套餐（全局）' : `编辑节点套餐 #${plan?.id || ''}`}</h3>
        <button class="btn small" id="admNodePlanCloseTop">关闭</button>
      </div>

      <div class="grid cols-2" style="margin-top:10px;">
        <div class="field"><label>套餐名称</label><input id="adm_np_name" value="${escapeHtml(plan?.name || '')}"></div>
        <div class="field"><label>发布者用户 ID（可空）</label><input id="adm_np_owner" type="number" value="${plan?.owner_user_id ?? ''}"></div>
        <div class="field"><label>可见方式</label>
          <select id="adm_np_visibility_scope">
            <option value="public">公开展示</option>
            <option value="link_only">仅专属链接购买</option>
            <option value="assigned_only">仅指定用户购买</option>
          </select>
        </div>
        <div class="field"><label>是否上架展示</label><select id="adm_np_show"><option value="1">是</option><option value="0">否</option></select></div>
        <div class="field"><label>是否允许购买</label><select id="adm_np_sell"><option value="1">是</option><option value="0">否</option></select></div>
        <div class="field"><label>是否允许续费</label><select id="adm_np_renew"><option value="1">是</option><option value="0">否</option></select></div>
        <div class="field"><label>显示顺序（数字越小越靠前）</label><input id="adm_np_sort" type="number" value="${plan?.sort ?? 0}"></div>
        <div class="field">
          <label>付费额度（GB）</label>
          <input id="adm_np_transfer" type="number" min="0" value="${plan?.paid_quota_gb ?? plan?.transfer_enable ?? 0}">
          <label class="row" style="margin-top:6px;align-items:center;gap:8px;">
            <input id="adm_np_unlimited_traffic" type="checkbox" ${initialUnlimitedTraffic ? 'checked' : ''}>
            <span class="muted">无限流量（购买后不限总流量）</span>
          </label>
        </div>
        <div class="field"><label>人数上限（空=不限）</label><input id="adm_np_capacity" type="number" min="0" value="${plan?.capacity_limit ?? ''}"></div>
        <div class="field"><label>限速（Mbps，可空）</label><input id="adm_np_speed" type="number" min="0" value="${plan?.speed_limit ?? ''}"></div>
        <div class="field"><label>设备数上限（可空）</label><input id="adm_np_device" type="number" min="0" value="${plan?.device_limit ?? ''}"></div>
        <div class="field"><label>最低用户等级（0-4）</label><input id="adm_np_min_trust" type="number" min="0" max="4" value="${plan?.min_trust_level ?? ''}"></div>
        <div class="field" style="grid-column: 1 / -1;" id="adm_np_assigned_users_wrap">
          <label>可购买用户 ID（仅指定用户模式生效，多个用逗号）</label>
          <input id="adm_np_access_user_ids" value="${escapeHtml(initialAccessUserIds)}" placeholder="例如 1,2,3">
        </div>
        <div class="field" style="grid-column: 1 / -1;" id="adm_np_share_token_wrap">
          <label>专属购买口令（仅专属链接模式，留空自动生成）</label>
          <input id="adm_np_share_token" value="${escapeHtml(initialShareToken)}" placeholder="例如 Abc12345xyz">
          <div class="row" style="margin-top:6px;">
            <input id="adm_np_share_link_preview" readonly value="${escapeHtml(buildNodePlanShareLink(initialShareToken))}" placeholder="保存后生成专属链接">
            <button class="btn small" id="adm_np_copy_share_link_btn" type="button">复制链接</button>
          </div>
        </div>
        <div class="field" style="grid-column: 1 / -1;">
          <label>节点（多选）</label>
          <div class="grid" style="grid-template-columns: repeat(2, minmax(0,1fr)); gap: 8px; max-height: 220px; overflow: auto;">
            ${normalizedNodeOptions.length
              ? normalizedNodeOptions.map((node) => {
                const checked = selectedNodeIdSet.has(node.id) ? 'checked' : '';
                const ownerText = node.owner_email || (node.user_id > 0 ? `UID ${node.user_id}` : '-');
                const nodeStatusText = (() => {
                  const onlineStatus = String(node?.online_status || '').toLowerCase();
                  const rawIsOnline = node?.is_online;
                  const isOnline = rawIsOnline === true || rawIsOnline === 1 || rawIsOnline === '1';
                  const isOffline = rawIsOnline === false || rawIsOnline === 0 || rawIsOnline === '0';
                  if (onlineStatus === 'online' || isOnline) return '在线';
                  if (onlineStatus === 'offline' || isOffline) return '离线';
                  return String(node?.status || '未知');
                })();
                return `<label class="notice" style="display:flex;gap:10px;align-items:center;">
                  <input type="checkbox" class="adm_np_node" value="${node.id}" ${checked}>
                  <span>${escapeHtml(node.name || ('节点 #' + node.id))} (#${node.id}) · ${escapeHtml(ownerText)} · ${escapeHtml(nodeStatusText)}</span>
                </label>`;
              }).join('')
              : '<div class="muted">暂无可选节点，请先创建节点。</div>'}
          </div>
        </div>
        <div class="field" style="grid-column: 1 / -1;"><label>描述</label><textarea id="adm_np_content" rows="3">${escapeHtml(plan?.content || '')}</textarea></div>
        <div class="field">
          <label>允许试用（免费额度）</label>
          <select id="adm_np_allow_trial">
            <option value="1">允许</option>
            <option value="0">不允许</option>
          </select>
        </div>
        <div class="field" style="grid-column: 1 / -1;">
          <label>试用免费额度（GB/月，按用户等级）</label>
          <div class="grid cols-2" id="adm_np_free_quota_wrap">
            ${renderFreeQuotaInputFields('adm_np_free_quota_', initialFreeQuota)}
          </div>
        </div>
      </div>

      <div class="field" style="margin-top:10px;">
        <label>价格（元）</label>
        <div class="grid cols-2">
          ${PLAN_PRICE_FIELDS.map((item) => {
            const hasPrice = Object.prototype.hasOwnProperty.call(initialPrices, item.key);
            const priceFen = Number(initialPrices[item.key] || 0);
            const priceYuan = hasPrice ? (priceFen / 100).toFixed(2) : '';
            return `
              <div class="field">
                <label>${item.label}</label>
                <input id="adm_np_price_${item.key}" type="number" min="0" step="0.01" value="${priceYuan}" placeholder="留空=不售卖">
              </div>
            `;
          }).join('')}
        </div>
      </div>

      <div class="row end" style="margin-top:10px;">
        <button class="btn" id="admNodePlanCloseBottom">取消</button>
        <button class="btn primary" id="admNodePlanSaveBtn">保存</button>
      </div>
    </div>
  `;

  document.body.appendChild(modal);

  const close = () => modal.remove();
  const closeTop = qs('#admNodePlanCloseTop', modal);
  const closeBottom = qs('#admNodePlanCloseBottom', modal);
  if (closeTop) closeTop.addEventListener('click', close);
  if (closeBottom) closeBottom.addEventListener('click', close);
  modal.addEventListener('click', (e) => {
    if (e.target === modal) close();
  });

  qs('#adm_np_show', modal).value = String(plan?.show ? 1 : 0);
  qs('#adm_np_sell', modal).value = String(plan?.sell ? 1 : 0);
  qs('#adm_np_renew', modal).value = String(plan?.renew ? 1 : 0);
  qs('#adm_np_allow_trial', modal).value = initialAllowTrial ? '1' : '0';
  qs('#adm_np_visibility_scope', modal).value = initialVisibility;

  const syncPlanVisibilityUi = () => {
    const visibilityScope = normalizePlanVisibilityScope(qs('#adm_np_visibility_scope', modal)?.value);
    const showSelect = qs('#adm_np_show', modal);
    const assignedWrap = qs('#adm_np_assigned_users_wrap', modal);
    const shareWrap = qs('#adm_np_share_token_wrap', modal);
    if (assignedWrap) assignedWrap.classList.toggle('hidden', visibilityScope !== 'assigned_only');
    if (shareWrap) shareWrap.classList.toggle('hidden', visibilityScope !== 'link_only');
    if (showSelect) {
      if (visibilityScope === 'public') {
        showSelect.disabled = false;
      } else {
        showSelect.value = '0';
        showSelect.disabled = true;
      }
    }
  };

  const syncTrialUi = () => {
    const allowTrial = qs('#adm_np_allow_trial', modal)?.value === '1';
    const freeQuotaWrap = qs('#adm_np_free_quota_wrap', modal);
    if (freeQuotaWrap) freeQuotaWrap.classList.toggle('hidden', !allowTrial);
  };

  const syncUnlimitedTrafficUi = () => {
    const isUnlimited = Boolean(qs('#adm_np_unlimited_traffic', modal)?.checked);
    const transferInput = qs('#adm_np_transfer', modal);
    if (!transferInput) return;

    if (isUnlimited) {
      transferInput.dataset.prevValue = transferInput.value;
      transferInput.value = '';
      transferInput.disabled = true;
      transferInput.placeholder = '无限流量';
    } else {
      transferInput.disabled = false;
      transferInput.placeholder = '';
      if (transferInput.value === '' && transferInput.dataset.prevValue !== undefined) {
        transferInput.value = transferInput.dataset.prevValue;
      }
    }
  };

  const syncSharePreview = () => {
    const tokenInput = qs('#adm_np_share_token', modal);
    const preview = qs('#adm_np_share_link_preview', modal);
    if (!preview) return;
    preview.value = buildNodePlanShareLink(tokenInput?.value || '');
  };

  qs('#adm_np_visibility_scope', modal)?.addEventListener('change', syncPlanVisibilityUi);
  qs('#adm_np_allow_trial', modal)?.addEventListener('change', syncTrialUi);
  qs('#adm_np_unlimited_traffic', modal)?.addEventListener('change', syncUnlimitedTrafficUi);
  qs('#adm_np_share_token', modal)?.addEventListener('input', syncSharePreview);
  qs('#adm_np_copy_share_link_btn', modal)?.addEventListener('click', async () => {
    try {
      await copyText(qs('#adm_np_share_link_preview', modal)?.value || '');
      alert('专属购买链接已复制');
    } catch (e) {
      alert(e.message || '复制失败');
    }
  });
  syncPlanVisibilityUi();
  syncTrialUi();
  syncUnlimitedTrafficUi();
  syncSharePreview();

  const saveBtn = qs('#admNodePlanSaveBtn', modal);
  if (saveBtn) {
    saveBtn.addEventListener('click', async () => {
      try {
        const prices = {};
        PLAN_PRICE_FIELDS.forEach((item) => {
          const raw = String(qs(`#adm_np_price_${item.key}`, modal)?.value || '').trim();
          if (!raw) return;
          const val = Number(raw);
          if (!Number.isFinite(val) || val < 0) {
            throw new Error(`${item.label}价格必须大于等于 0`);
          }
          prices[item.key] = Number(val.toFixed(2));
        });
        if (!Object.keys(prices).length) {
          throw new Error('至少设置一个购买价格');
        }

        const ownerRaw = String(qs('#adm_np_owner', modal)?.value || '').trim();
        const minTrustRaw = String(qs('#adm_np_min_trust', modal)?.value || '').trim();
        const nodeIds = qsa('.adm_np_node', modal)
          .filter((input) => input.checked)
          .map((input) => Number(input.value))
          .filter((value) => Number.isFinite(value) && value > 0);
        const allowTrial = qs('#adm_np_allow_trial', modal)?.value === '1';
        const isUnlimitedTraffic = Boolean(qs('#adm_np_unlimited_traffic', modal)?.checked);
        const visibilityScope = normalizePlanVisibilityScope(qs('#adm_np_visibility_scope', modal)?.value);
        const freeQuota = allowTrial ? readFreeQuotaInputs('adm_np_free_quota_', modal) : null;
        const accessUserIds = visibilityScope === 'assigned_only'
          ? parseIdListInput(qs('#adm_np_access_user_ids', modal)?.value || '')
          : [];
        const shareTokenRaw = visibilityScope === 'link_only'
          ? String(qs('#adm_np_share_token', modal)?.value || '').trim()
          : '';

        const payload = {
          name: String(qs('#adm_np_name', modal)?.value || '').trim(),
          owner_user_id: ownerRaw ? Number(ownerRaw) : null,
          min_trust_level: minTrustRaw === '' ? null : Number(minTrustRaw),
          node_ids: Array.from(new Set(nodeIds)),
          prices,
          show: qs('#adm_np_show', modal)?.value === '1',
          sell: qs('#adm_np_sell', modal)?.value === '1',
          renew: qs('#adm_np_renew', modal)?.value === '1',
          sort: Number(qs('#adm_np_sort', modal)?.value || 0),
          transfer_enable: isUnlimitedTraffic ? 0 : Number(qs('#adm_np_transfer', modal)?.value || 0),
          is_unlimited_traffic: isUnlimitedTraffic,
          capacity_limit: String(qs('#adm_np_capacity', modal)?.value || '').trim() === '' ? null : Number(qs('#adm_np_capacity', modal)?.value),
          speed_limit: String(qs('#adm_np_speed', modal)?.value || '').trim() === '' ? null : Number(qs('#adm_np_speed', modal)?.value),
          device_limit: String(qs('#adm_np_device', modal)?.value || '').trim() === '' ? null : Number(qs('#adm_np_device', modal)?.value),
          content: String(qs('#adm_np_content', modal)?.value || ''),
          allow_trial: allowTrial,
          free_quota_gb_by_trust_level: freeQuota,
          visibility_scope: visibilityScope,
          access_user_ids: accessUserIds,
          share_token: shareTokenRaw || null,
        };

        if (!payload.name) throw new Error('套餐名称不能为空');
        if (!payload.node_ids.length) throw new Error('至少选择一个节点');
        if (visibilityScope === 'assigned_only' && !accessUserIds.length) {
          throw new Error('指定用户模式下，至少填写一个可购买用户 ID');
        }

        if (typeof onSubmit === 'function') {
          await onSubmit(payload);
        }
        close();
      } catch (e) {
        alert(e.message || '保存失败');
      }
    });
  }
}

function showTcpingAgentCreateModal({ defaultName = '', onSubmit } = {}) {
  const modal = document.createElement('div');
  modal.id = 'tcpingAgentCreateModal';
  modal.className = 'modal-overlay';
  modal.innerHTML = html`
    <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="tcpingAgentCreateTitle" style="max-width: 720px;">
      <div class="modal-head">
        <h3 id="tcpingAgentCreateTitle">新增 TCPing Agent</h3>
        <button class="btn small" id="tcpingAgentCreateCloseTop" type="button">关闭</button>
      </div>
      <div class="muted">Agent 用于 TCPing 探测，强制填写归属地区。中国大陆必须填写到省份。</div>
      <div class="grid cols-2" style="margin-top: 12px;">
        <div class="field">
          <label>Agent 名称</label>
          <input id="tcpingAgentName" value="${escapeHtml(defaultName || '')}" placeholder="例如 agent-hk-01">
        </div>
        <div class="field">
          <label>归属国家 / 地区</label>
          <input id="tcpingAgentLocation" list="tcpingAgentLocationOptions" placeholder="必选，可搜索，例如 CN | China Mainland 中国大陆" required>
        </div>
        <div class="field hidden" id="tcpingAgentProvinceWrap" style="grid-column: 1 / -1;">
          <label>中国大陆省份归属</label>
          <input id="tcpingAgentProvince" list="cnProvinceOptions" placeholder="必填，例如 广东 / 上海">
        </div>
      </div>
      ${renderTcpingAgentLocationDatalist()}
      ${renderCnProvinceDatalist()}
      <div class="row end" style="margin-top: 12px;">
        <button class="btn" id="tcpingAgentCreateCloseBottom" type="button">取消</button>
        <button class="btn primary" id="tcpingAgentCreateBtn" type="button">创建</button>
      </div>
    </div>
  `;

  document.body.appendChild(modal);

  const close = () => modal.remove();
  qs('#tcpingAgentCreateCloseTop', modal)?.addEventListener('click', close);
  qs('#tcpingAgentCreateCloseBottom', modal)?.addEventListener('click', close);
  modal.addEventListener('click', (e) => {
    if (e.target === modal) close();
  });

  const nameInput = qs('#tcpingAgentName', modal);
  const locationInput = qs('#tcpingAgentLocation', modal);
  const provinceWrap = qs('#tcpingAgentProvinceWrap', modal);
  const provinceInput = qs('#tcpingAgentProvince', modal);

  const syncProvinceUi = () => {
    if (!locationInput || !provinceWrap || !provinceInput) return;
    const parsed = parseTcpingAgentLocationSelection(locationInput.value);
    const code = String(parsed?.location_code || '').toUpperCase();
    const isCn = code === 'CN' || String(locationInput.value || '').trim().toUpperCase().startsWith('CN');
    provinceWrap.classList.toggle('hidden', !isCn);
    provinceInput.required = isCn;
  };
  if (locationInput) {
    locationInput.addEventListener('input', syncProvinceUi);
    locationInput.addEventListener('change', syncProvinceUi);
  }
  syncProvinceUi();

  const createBtn = qs('#tcpingAgentCreateBtn', modal);
  if (createBtn) {
    createBtn.addEventListener('click', async () => {
      try {
        const name = String(nameInput?.value || '').trim();
        if (!name) throw new Error('请输入 Agent 名称');
        const location = readTcpingAgentLocationSelection('tcpingAgentLocation');
        const code = String(location.location_code || '').toUpperCase();
        const province = String(provinceInput?.value || '').trim();
        if (code === 'CN' && !province) {
          throw new Error('中国大陆地区的探针必须填写省份归属');
        }

        const payload = {
          name,
          location_code: location.location_code,
          location_name: location.location_name,
          location_province: code === 'CN' ? province : null,
        };

        if (typeof onSubmit === 'function') {
          await onSubmit(payload);
        }
        close();
      } catch (e) {
        alert(e.message || '创建失败');
      }
    });
  }
}

function pillStatus(node) {
  const onlineStatus = String(node?.online_status || '').toLowerCase();
  const rawIsOnline = node?.is_online;
  const isOnline = rawIsOnline === true || rawIsOnline === 1 || rawIsOnline === '1';
  const isOffline = rawIsOnline === false || rawIsOnline === 0 || rawIsOnline === '0';

  if (onlineStatus === 'online' || isOnline) {
    return `<span class="pill ok">在线</span>`;
  }
  if (onlineStatus === 'offline' || isOffline) {
    return `<span class="pill bad">离线</span>`;
  }

  const status = String(node?.status || 'unknown').toLowerCase();
  if (status === 'active') return `<span class="pill ok">可用</span>`;
  if (status === 'deploying') return `<span class="pill warn">部署中</span>`;
  if (status === 'maintenance') return `<span class="pill warn">维护中</span>`;
  if (status === 'inactive') return `<span class="pill">未启用</span>`;
  return `<span class="pill">${escapeHtml(String(node?.status || '未知'))}</span>`;
}

async function loadMe() {
  if (!store.auth) return null;
  const res = await apiFetch('/api/v1/user/me');
  store.me = res?.data || null;
  syncAuthNav();

  return store.me;
}

async function renderLogin(errorText = '', activePanel = 'login', extraOptions = {}) {
  syncAuthNav();

  const route = parseHashRoute(location.hash || '#/login');
  let config = loginCaptchaState.config || guestCommConfigCache || {};
  if (!config || !Object.keys(config).length) {
    try {
      config = await loadGuestCommConfig();
    } catch (_) {
      config = guestCommConfigCache || {};
    }
  }
  loginCaptchaState.config = config || {};

  const inviteCode = String(extraOptions?.inviteCode ?? route.query.invite_code ?? '').trim();
  const allowEmailRegister = Number(config?.allow_email_register || 0) === 1;
  const allowOauthRegister = Number(config?.allow_oauth_register || 0) === 1;
  const oauthEnabled = Number(config?.oauth_linux_do_enable || 0) === 1;
  const isEmailVerifyEnabled = Number(config?.is_email_verify || 0) === 1;
  const isInviteForced = Number(config?.is_invite_force || 0) === 1;
  const desiredPanel = String(activePanel || '').toLowerCase() === 'register' ? 'register' : 'login';
  const panel = desiredPanel === 'register' && allowEmailRegister ? 'register' : 'login';
  const isRegister = panel === 'register';
  const oauthUrl = buildOAuthLinuxDoUrl(inviteCode);
  const registerBlockedText = !allowEmailRegister
    ? (oauthEnabled && allowOauthRegister
      ? '邮箱注册已关闭，请使用 Linux DO 完成首次注册。'
      : '当前站点已关闭新用户注册。')
    : '';

  setView(html`
    <div class="card auth-card">
      <div class="auth-header">
        <h2>${isRegister ? '注册' : '登录'}</h2>
        <div class="auth-switcher">
          <button class="btn ${isRegister ? '' : 'primary'}" id="switchToLogin" type="button">登录</button>
          ${allowEmailRegister
            ? `<button class="btn ${isRegister ? 'primary' : ''}" id="switchToRegister" type="button">注册</button>`
            : ''}
        </div>
      </div>

      ${errorText ? `<div class="notice error" style="margin-top:10px;">${escapeHtml(errorText)}</div>` : ''}
      ${registerBlockedText ? `<div class="notice" style="margin-top:10px;">${escapeHtml(registerBlockedText)}</div>` : ''}
      ${inviteCode
        ? `<div class="notice" style="margin-top:10px;">当前邀请码：<code>${escapeHtml(inviteCode)}</code>${isRegister ? '，注册成功后会自动绑定邀请关系。' : '，如需注册请切到注册面板或直接使用 Linux DO。'}</div>`
        : ''}
      ${isInviteForced && !inviteCode
        ? '<div class="notice" style="margin-top:10px;">当前站点要求必须通过邀请链接注册。</div>'
        : ''}

      <div class="grid" style="margin-top: 10px;">
        ${isRegister
          ? html`
            <div class="field">
              <label>邮箱</label>
              <input id="regEmail" type="email" autocomplete="username" placeholder="you@example.com">
            </div>
            <div class="field">
              <label>密码</label>
              <input id="regPassword" type="password" autocomplete="new-password" placeholder="••••••••">
            </div>
            ${isEmailVerifyEnabled
              ? html`
                <div class="field">
                  <label>邮箱验证码</label>
                  <div class="row" style="gap:8px;">
                    <input id="regEmailCode" placeholder="请输入 6 位验证码">
                    <button class="btn" id="sendEmailCodeBtn" type="button">发送验证码</button>
                  </div>
                </div>
              `
              : ''}
            <div class="field">
              <label>邀请码${isInviteForced ? '（必填）' : '（可空）'}</label>
              <input id="regInviteCode" value="${escapeHtml(inviteCode)}" placeholder="例如 ABCD1234">
            </div>
            <div class="field hidden" id="regTurnstileWrap">
              <label>人机验证</label>
              <div id="regTurnstileWidget"></div>
            </div>
            <div class="row">
              <button class="btn primary" id="regBtn">立即注册</button>
              ${oauthEnabled && allowOauthRegister
                ? `<a class="btn" id="oauthRegisterBtn" href="${escapeHtml(oauthUrl)}">Linux DO 注册</a>`
                : ''}
              <button class="btn" id="backToLoginBtn" type="button">返回登录</button>
            </div>
            <div class="muted">注册成功后会自动登录并进入系统。</div>
          `
          : html`
            <div class="field">
              <label>邮箱</label>
              <input id="loginEmail" type="email" autocomplete="username" placeholder="you@example.com">
            </div>
            <div class="field">
              <label>密码</label>
              <input id="loginPassword" type="password" autocomplete="current-password" placeholder="••••••••">
            </div>
            <div class="field hidden" id="loginTurnstileWrap">
              <label>人机验证</label>
              <div id="loginTurnstileWidget"></div>
            </div>
            <div class="row">
              <button class="btn primary" id="loginBtn">邮箱登录</button>
              ${oauthEnabled
                ? `<a class="btn" id="oauthLoginBtn" href="${escapeHtml(oauthUrl)}">${allowOauthRegister ? 'Linux DO 登录 / 注册' : 'Linux DO 登录'}</a>`
                : ''}
            </div>
            <div class="muted">Linux DO 登录成功后会自动返回本页面并写入本地登录态。</div>
          `
        }
        <div class="notice hidden" id="loginCaptchaHint"></div>
        <div class="notice hidden" id="loginPowHint"></div>
      </div>
    </div>
  `);

  setupLoginCaptchaUi(panel).catch(() => {});
  ensurePowProofForMode(panel)
    .catch((error) => {
      setPowHint(error?.message || '防刷验证准备失败，请刷新后重试。', true);
    })
    .finally(() => {
      updateAuthSubmitAvailability(panel);
    });

  const getCurrentInviteQuery = () => {
    const currentInviteCode = String(qs('#regInviteCode')?.value || inviteCode).trim();
    return currentInviteCode ? { invite_code: currentInviteCode } : {};
  };
  const switchToLoginBtn = qs('#switchToLogin');
  const switchToRegisterBtn = qs('#switchToRegister');
  if (switchToLoginBtn) {
    switchToLoginBtn.addEventListener('click', () => {
      location.hash = buildHashPath('/login', getCurrentInviteQuery());
    });
  }
  if (switchToRegisterBtn) {
    switchToRegisterBtn.addEventListener('click', () => {
      location.hash = buildHashPath('/login', { ...getCurrentInviteQuery(), register: '1' });
    });
  }

  const backToLoginBtn = qs('#backToLoginBtn');
  if (backToLoginBtn) {
    backToLoginBtn.addEventListener('click', () => {
      location.hash = buildHashPath('/login', getCurrentInviteQuery());
    });
  }

  const sendEmailCodeBtn = qs('#sendEmailCodeBtn');
  const syncOauthRegisterHref = () => {
    const oauthRegisterBtn = qs('#oauthRegisterBtn');
    if (!oauthRegisterBtn) return;
    const latestInviteCode = String(qs('#regInviteCode')?.value || inviteCode).trim();
    oauthRegisterBtn.setAttribute('href', buildOAuthLinuxDoUrl(latestInviteCode));
  };
  qs('#regInviteCode')?.addEventListener('input', syncOauthRegisterHref);
  syncOauthRegisterHref();

  if (sendEmailCodeBtn) {
    sendEmailCodeBtn.addEventListener('click', async () => {
      try {
        const email = String(qs('#regEmail')?.value || '').trim();
        if (!email) {
          throw new Error('请先输入邮箱地址');
        }

        const payload = { email };
        Object.assign(payload, await buildCaptchaPayloadForMode('register', config));

        await apiFetch('/api/v1/passport/comm/sendEmailVerify', { method: 'POST', body: payload });
        alert('邮箱验证码已发送，请查看收件箱。');

        resetCaptchaUiForMode('register', config);
      } catch (e) {
        alert(e.message || '发送验证码失败');
      }
    });
  }

  const loginBtn = qs('#loginBtn');
  if (loginBtn) {
    loginBtn.addEventListener('click', async () => {
      const email = String(qs('#loginEmail')?.value || '').trim();
      const password = String(qs('#loginPassword')?.value || '');
      try {
        if (!loginCaptchaState.config) {
          try {
            loginCaptchaState.config = await loadGuestCommConfig();
          } catch (_) {}
        }

        const latestConfig = loginCaptchaState.config || {};
        const payload = { email, password };
        Object.assign(payload, await buildCaptchaPayloadForMode('login', latestConfig));
        if (isPowEnabled(latestConfig)) {
          const proof = consumePowProofByMode('login');
          if (!proof) {
            throw new Error('防刷验证尚未准备完成，请稍候再试。');
          }
          payload.pow_id = proof.id;
          payload.pow_nonce = proof.nonce;
          payload.pow_hash = proof.hash;
          payload.pow_token = proof.token;
        }

        const res = await apiFetch('/api/v1/passport/auth/login', { method: 'POST', body: payload });
        store.auth = res?.data?.auth_data || '';
        store.legacyToken = res?.data?.token || '';
        store.me = null;
        await loadMe();
        const redirectHash = consumePostLoginRedirect();
        location.hash = redirectHash || '#/dashboard';
      } catch (e) {
        resetCaptchaUiForMode('login', loginCaptchaState.config || {});
        renderLogin(e.message || '登录失败', 'login', { inviteCode });
      }
    });
  }

  const regBtn = qs('#regBtn');
  if (regBtn) {
    regBtn.addEventListener('click', async () => {
      const email = String(qs('#regEmail')?.value || '').trim();
      const password = String(qs('#regPassword')?.value || '');
      const currentInviteCode = String(qs('#regInviteCode')?.value || inviteCode).trim();
      const emailCode = String(qs('#regEmailCode')?.value || '').trim();
      try {
        if (!loginCaptchaState.config) {
          try {
            loginCaptchaState.config = await loadGuestCommConfig();
          } catch (_) {}
        }

        const latestConfig = loginCaptchaState.config || {};
        const payload = {
          email,
          password,
          invite_code: currentInviteCode,
        };

        if (isEmailVerifyEnabled && emailCode) {
          payload.email_code = emailCode;
        }

        Object.assign(payload, await buildCaptchaPayloadForMode('register', latestConfig));
        if (isPowEnabled(latestConfig)) {
          const proof = consumePowProofByMode('register');
          if (!proof) {
            throw new Error('防刷验证尚未准备完成，请稍候再试。');
          }
          payload.pow_id = proof.id;
          payload.pow_nonce = proof.nonce;
          payload.pow_hash = proof.hash;
          payload.pow_token = proof.token;
        }

        const res = await apiFetch('/api/v1/passport/auth/register', { method: 'POST', body: payload });
        store.auth = res?.data?.auth_data || '';
        store.legacyToken = res?.data?.token || '';
        store.me = null;
        await loadMe();
        const redirectHash = consumePostLoginRedirect();
        location.hash = redirectHash || '#/dashboard';
      } catch (e) {
        resetCaptchaUiForMode('register', loginCaptchaState.config || {});
        renderLogin(e.message || '注册失败', 'register', { inviteCode: currentInviteCode });
      }
    });
  }

  updateAuthSubmitAvailability(panel);
}

async function renderDashboard() {
  if (!requireAuth()) return;

  const me = await loadMe();
  const info = await apiFetch('/api/v1/user/info');
  const subscribe = await apiFetch('/api/v1/user/getSubscribe');
  const planQuota = await apiFetch('/api/v1/user/plan/quota');
  const apiKey = await apiFetch('/api/v1/user/api-key');
  const accessStats = await apiFetch('/api/v1/user/access/stats');

  const userInfo = info?.data || {};
  const subInfo = subscribe?.data || {};
  const quotaInfo = planQuota?.data || {};
  const keyInfo = apiKey?.data || {};
  const stats = accessStats?.data || {};
  const roleDuty = me?.is_super_admin
    ? '站长：负责站点设置和全局管理'
    : (me?.is_admin ? '管理员：协助处理节点与工单' : '用户：提供节点并使用订阅');

  const linuxLabel = me?.is_linux_do_user ? `<span class="pill ok">Linux DO Connect</span>` : '';

  setView(html`
    <div class="grid cols-2">
      <div class="card">
        <h2>账户</h2>
        <div class="kvs">
          <div class="k">邮箱</div><div class="v">${escapeHtml(userInfo.email || '-')}</div>
          <div class="k">UUID</div><div class="v">${escapeHtml(userInfo.uuid || '-')}</div>
          <div class="k">用户等级</div><div class="v">${me?.trust_level ?? '-'}</div>
          <div class="k">第三方用户名</div><div class="v">${escapeHtml(me?.linux_do_username || '-')}</div>
          <div class="k">身份</div><div class="v">
            ${linuxLabel}
            ${me?.is_super_admin ? `<span class="pill warn">站长</span>` : (me?.is_admin ? `<span class="pill">管理员</span>` : `<span class="pill">用户</span>`)}
          </div>
          <div class="k">角色说明</div><div class="v">${roleDuty}</div>
          <div class="k">到期时间</div><div class="v">${subInfo.expired_at ? formatTs(subInfo.expired_at) : '-'}</div>
          <div class="k">订阅链接</div><div class="v"><button class="btn small" id="copySubBtnDash">复制订阅链接</button></div>
        </div>
      </div>
      <div class="card">
        <h2>API Key</h2>
        <div class="kvs">
          <div class="k">是否已生成</div><div class="v">${keyInfo.has_api_key ? '是' : '否'}</div>
          <div class="k">当前 Key</div><div class="v">${escapeHtml(keyInfo.api_key || '-')}</div>
          <div class="k">创建时间</div><div class="v">${escapeHtml(keyInfo.created_at || '-')}</div>
        </div>
        <div class="row end" style="margin-top: 12px;">
          <button class="btn" id="resetApiKeyBtn">${keyInfo.has_api_key ? '重置 Key' : '生成 Key'}</button>
        </div>
      </div>
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>套餐额度使用</h2>
        <div class="muted">不同套餐独立扣费，这里按套餐实例展示</div>
      </div>
      <div class="kvs" style="margin-top:10px;">
        <div class="k">总额度</div><div class="v">${formatTrafficDisplay(quotaInfo.total_allowance_kb || 0, Boolean(quotaInfo.has_unlimited_traffic))}</div>
        <div class="k">已使用</div><div class="v">${formatTrafficKb(quotaInfo.total_used_kb || 0)}</div>
        <div class="k">剩余额度</div><div class="v">${formatTrafficDisplay(quotaInfo.total_remaining_kb || 0, Boolean(quotaInfo.has_unlimited_traffic))}</div>
      </div>
      ${renderPlanQuotaTable(quotaInfo)}
    </div>

    <div class="grid cols-3" style="margin-top: 12px;">
      <div class="card">
        <h3>节点统计</h3>
        <div class="kvs">
          <div class="k">我提供的节点</div><div class="v">${stats.owned_nodes ?? '-'}</div>
          <div class="k">可用节点总数</div><div class="v">${stats.accessible_nodes ?? '-'}</div>
          <div class="k">被分享的节点</div><div class="v">${stats.shared_nodes ?? '-'}</div>
        </div>
      </div>
      <div class="card">
        <h3>并发 IP 限制</h3>
        <div class="muted">同用户多 IP 并发限制（默认 3，0 代表使用默认）。</div>
        <div class="kvs" style="margin-top: 10px;">
          <div class="k">我的限制</div><div class="v">${(Number(me?.concurrent_ip_limit) > 0) ? Number(me?.concurrent_ip_limit) : 3}</div>
          <div class="k">默认策略</div><div class="v">3</div>
        </div>
      </div>
      <div class="card">
        <h3>快速入口</h3>
        <div class="row">
          <a class="btn primary" href="#/nodes">管理节点</a>
          <a class="btn" href="#/audit">审计日志</a>
        </div>
      </div>
    </div>
  `);

  qs('#resetApiKeyBtn').addEventListener('click', async () => {
    try {
      const hasApiKey = Boolean(keyInfo.has_api_key);
      const endpoint = hasApiKey ? '/api/v1/user/api-key/reset' : '/api/v1/user/api-key/generate';
      await apiFetch(endpoint, { method: 'POST', body: {} });
      alert(hasApiKey ? 'API Key 已重置' : 'API Key 已生成');
      await renderDashboard();
    } catch (e) {
      alert(e.message || '操作失败');
    }
  });

  const copySubBtnDash = qs('#copySubBtnDash');
  if (copySubBtnDash) {
    copySubBtnDash.addEventListener('click', async () => {
      try {
        await copyText(subInfo.subscribe_url || '');
        alert('订阅链接已复制');
      } catch (e) {
        alert(e.message || '复制失败');
      }
    });
  }
}

async function renderNodes() {
  if (!requireAuth()) return;
  await loadMe();

  let agents = [];
  let agentErr = '';
  const [owned, accessible, protocolOptions] = await Promise.all([
    apiFetch('/api/v1/user/server-nodes'),
    apiFetch('/api/v1/user/accessible-nodes'),
    loadNodeProtocolOptions(),
    apiFetch('/api/v1/user/tcping/agents')
      .then((res) => { agents = asArray(res?.data); })
      .catch((e) => {
        agentErr = e.message || '加载 TCPing Agent 失败';
        agents = [];
      }),
  ]);

  const protocolMap = Object.fromEntries(protocolOptions.map((item) => [item.value, item.label]));
  const ownedNodes = asArray(owned?.data);
  const accessibleNodes = asArray(accessible?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <h2>TCPing Agent 管理</h2>
          <div class="muted">独立部署在探测服务器上，不和 V2bX 混装。创建后复制一行命令即可接入。</div>
        </div>
        <button class="btn primary" id="newTcpingAgentBtn">新增 Agent</button>
      </div>
      ${agentErr ? `<div class="notice error" style="margin-top:10px;">${escapeHtml(agentErr)}</div>` : ''}
      ${agents.length ? html`
        <table class="table" style="margin-top:10px;">
          <thead>
            <tr>
              <th>ID</th><th>名称</th><th>归属</th><th>Token</th><th>状态</th><th>最后心跳</th><th>最后同步</th><th></th>
            </tr>
          </thead>
          <tbody>
            ${agents.map((agent) => `
              <tr>
                <td>${agent.id}</td>
                <td>${escapeHtml(agent.name || '-')}</td>
                <td class="muted">${escapeHtml(agent.location_display || '未设置')}</td>
                <td><code>${escapeHtml(maskToken(agent.token))}</code></td>
                <td><span class="pill ok">统一启用</span></td>
                <td class="muted">${escapeHtml(formatTs(agent.last_heartbeat_at))}</td>
                <td class="muted">${escapeHtml(formatTs(agent.last_sync_at))}</td>
                <td class="row end">
                  <button class="btn small" data-agent-action="install" data-id="${agent.id}">复制安装命令</button>
                  <button class="btn small" data-agent-action="rotate" data-id="${agent.id}">轮换 Token</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      ` : '<div class="notice" style="margin-top:10px;">还没有 TCPing Agent。建议至少部署 1 个，节点监控和被墙提醒才会开始工作。</div>'}
    </div>

    <div class="card" style="margin-top: 12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <h2>我的节点</h2>
          <div class="muted">创建时已经预设常见参数，地区必选，其余大多数配置都折叠在高级设置里。</div>
        </div>
        <button class="btn primary" id="newNodeBtn">新增节点</button>
      </div>
      ${ownedNodes.length ? html`
        <table class="table" style="margin-top:10px;">
          <thead>
            <tr>
              <th>ID</th><th>节点</th><th>协议 / 入口</th><th>状态</th><th>节点流量</th><th>扣费倍率</th><th>TCPing</th><th></th>
            </tr>
          </thead>
          <tbody>
            ${ownedNodes.map((node) => `
              <tr>
                <td>${node.id}</td>
                <td>
                  <div>${escapeHtml(node.name || '-')}</div>
                  <div class="muted">${escapeHtml(node.location_name || '未设置地区')}</div>
                </td>
                <td>
                  <div><span class="pill">${escapeHtml(protocolDisplayName(node.protocol, protocolMap))}</span></div>
                  <div class="muted" style="margin-top:4px;">${escapeHtml(node.host || '-')}</div>
                  <div class="muted">访问 ${escapeHtml(String(node.port ?? '-'))} / 服务 ${escapeHtml(String(node.effective_service_port ?? node.service_port ?? node.port ?? '-'))}</div>
                </td>
                <td>
                  <div>${pillStatus(node)}</div>
                  <div class="muted" style="margin-top:4px;">在线用户 ${escapeHtml(String(node.online_users ?? 0))}</div>
                </td>
                <td>${renderNodeTrafficCell(node)}</td>
                <td><span class="pill">x${escapeHtml(String(Number(node.traffic_multiplier || 1).toFixed(2)))}</span></td>
                <td>${renderNodeTcpingCell(node)}</td>
                <td class="row end">
                  <button class="btn small" data-action="monitor" data-id="${node.id}">监控</button>
                  <button class="btn small" data-action="edit" data-id="${node.id}">编辑</button>
                  ${isV2bxDeployableProtocol(node.protocol)
                    ? `<button class="btn small" data-action="deploy" data-id="${node.id}">部署</button>`
                    : `<button class="btn small" data-action="deploy-unsupported" data-id="${node.id}">部署不可用</button>`
                  }
                  ${isV2bxDeployableProtocol(node.protocol) && node.v2bx_token_configured
                    ? `<button class="btn small" data-action="rotate-v2bx" data-id="${node.id}">轮换凭据</button>`
                    : ''}
                  <button class="btn small" data-action="access" data-id="${node.id}">分享/权限</button>
                  <button class="btn small" data-action="audit" data-id="${node.id}">审计规则</button>
                  <button class="btn small danger" data-action="delete" data-id="${node.id}">删除</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      ` : '<div class="notice" style="margin-top:10px;">你还没有创建节点。</div>'}
    </div>

    <div class="card" style="margin-top: 12px;">
      <h2>我可用的节点（含他人分享）</h2>
      <div class="muted">这里只展示已授权且可下发给你的节点，未上线节点不会出现在订阅下发结果中。</div>
      ${accessibleNodes.length ? html`
        <table class="table" style="margin-top: 10px;">
          <thead>
            <tr>
              <th>ID</th><th>节点</th><th>Owner</th><th>协议</th><th>状态</th><th>套餐扣费倍率</th><th>TCPing</th><th></th>
            </tr>
          </thead>
          <tbody>
            ${accessibleNodes.map((node) => `
              <tr>
                <td>${node.id}</td>
                <td>
                  <div>${escapeHtml(node.name || '-')}</div>
                  <div class="muted">${escapeHtml(node.location_name || '未设置地区')}</div>
                </td>
                <td class="muted">${escapeHtml(node.owner_email || String(node.user_id || '-'))}</td>
                <td><span class="pill">${escapeHtml(protocolDisplayName(node.protocol, protocolMap))}</span></td>
                <td>${pillStatus(node)}</td>
                <td><span class="pill">x${escapeHtml(String(Number(node.traffic_multiplier || 1).toFixed(2)))}</span></td>
                <td>${renderNodeTcpingCell(node)}</td>
                <td class="row end">
                  <button class="btn small" data-action="monitor" data-id="${node.id}">监控</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      ` : '<div class="notice" style="margin-top:10px;">当前没有可访问的共享节点。</div>'}
    </div>
  `);

  qs('#newNodeBtn').addEventListener('click', () => {
    location.hash = '#/nodes/new';
  });

  const newTcpingAgentBtn = qs('#newTcpingAgentBtn');
  if (newTcpingAgentBtn) {
    newTcpingAgentBtn.addEventListener('click', () => {
      showTcpingAgentCreateModal({
        defaultName: `agent-${Date.now()}`,
        onSubmit: async (payload) => {
          await apiFetch('/api/v1/user/tcping/agents', { method: 'POST', body: payload });
          await renderNodes();
        },
      });
    });
  }

  qsa('button[data-agent-action]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const action = String(btn.getAttribute('data-agent-action') || '');
      const id = btn.getAttribute('data-id');
      if (!id) return;

      try {
        if (action === 'install') {
          const res = await apiFetch(`/api/v1/user/tcping/agents/${id}/install-command`);
          const command = String(res?.data?.command || '').trim();
          if (!command) throw new Error('未生成安装命令');
          await copyText(command);
          alert('安装命令已复制。请在独立探测机上执行，不要和 V2bX 混装。');
          return;
        }

        if (action === 'rotate') {
          if (!confirm('确认轮换这个 Agent 的 Token？轮换后旧实例需要重新部署。')) return;
          await apiFetch(`/api/v1/user/tcping/agents/${id}/rotate-token`, { method: 'POST', body: {} });
          await renderNodes();
          return;
        }
      } catch (e) {
        alert(e.message || '操作失败');
      }
    });
  });

  qsa('button[data-action]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const action = btn.getAttribute('data-action');
      const id = btn.getAttribute('data-id');
      if (!id) return;
      try {
        if (action === 'monitor') {
          location.hash = `#/nodes/${id}/monitor`;
          return;
        }
        if (action === 'edit') {
          location.hash = `#/nodes/edit/${id}`;
          return;
        }
        if (action === 'deploy-unsupported') {
          alert('该协议暂不支持 V2bX 一键部署。请改用 vmess/vless/trojan/shadowsocks/hysteria/hysteria2/tuic/anytls。');
          return;
        }
        if (action === 'deploy') {
          const node = ownedNodes.find((n) => String(n.id) === String(id)) || {};
          if (!isV2bxDeployableProtocol(node?.protocol)) {
            alert('该协议暂不支持 V2bX 一键部署。');
            return;
          }
          const deployRes = await apiFetch(`/api/v1/user/server-nodes/${id}/deploy`, { method: 'POST' });

          let commandData = {};
          try {
            const commandRes = await apiFetch(`/api/v1/user/server-nodes/${id}/deploy-command`, { method: 'POST', body: {} });
            commandData = commandRes?.data || {};
          } catch (_) {}
          await renderNodes();
          showDeployInfoModal({
            nodeId: id,
            nodeName: node?.name || '',
            panelUrl: commandData?.panel_url || location.origin,
            command: commandData?.command || ''
          });

          if (deployRes?.message) {
            alert(deployRes.message);
          }
          return;
        }
        if (action === 'rotate-v2bx') {
          if (!confirm('确认轮换这个节点的 V2bX 凭据？旧实例会立即失效，请随后执行新的部署命令。')) return;
          const commandRes = await apiFetch(`/api/v1/user/server-nodes/${id}/deploy-command/rotate-token`, { method: 'POST', body: {} });
          const command = String(commandRes?.data?.command || '').trim();
          if (!command) throw new Error('未生成新的部署命令');
          await copyText(command);
          alert('新部署命令已复制，旧 V2bX 凭据已失效。');
          await renderNodes();
          return;
        }
        if (action === 'delete') {
          if (!confirm('确认删除该节点？')) return;
          await apiFetch(`/api/v1/user/server-nodes/${id}`, { method: 'DELETE' });
          await renderNodes();
          return;
        }
        if (action === 'access') {
          location.hash = `#/nodes/${id}/access`;
          return;
        }
        if (action === 'audit') {
          location.hash = `#/nodes/${id}/audit-rules`;
        }
      } catch (e) {
        alert(e.message || '操作失败');
      }
    });
  });
}

async function renderNodeMonitor(nodeId, rangeHours = 24, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const hours = Math.max(1, Math.min(24 * 30, Number(rangeHours) || 24));
  let payload = null;
  try {
    const res = await apiFetch(`/api/v1/user/server-nodes/${nodeId}/tcping?hours=${hours}`);
    payload = res?.data || null;
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '节点监控读取失败')}</div>`);
    return;
  }

  const node = payload?.node || {};
  const samples = asArray(payload?.samples);
  const alerts = asArray(payload?.alerts);
  const agentMeta = asArray(payload?.agents);
  const stats = payload?.stats || {};
  const tcpingStatusKey = String(node?.tcping_status || 'unknown').toLowerCase();
  const isTcpingUnsupported = tcpingStatusKey === 'unsupported';
  const tcpingSeries = buildTcpingSeriesByRegion(samples, agentMeta);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <h2>节点监控：${escapeHtml(node.name || `#${nodeId}`)}</h2>
          <div class="muted">${escapeHtml(node.location_name || '未设置地区')} · ${escapeHtml(protocolDisplayName(node.protocol || ''))}</div>
        </div>
        <div class="row">
          ${TCPING_MONITOR_RANGE_OPTIONS.map((item) => `
            <a class="btn ${hours === item.hours ? 'primary' : ''}" href="#/nodes/${encodeURIComponent(nodeId)}/monitor?hours=${item.hours}">${escapeHtml(item.label)}</a>
          `).join('')}
          <a class="btn" href="#/nodes">返回节点列表</a>
        </div>
      </div>
      ${errorText ? `<div class="notice error" style="margin-top:10px;">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid cols-3" style="margin-top:12px;">
        <div class="card">
          <h3>当前状态</h3>
          <div>${renderTcpingStatusPill(node.tcping_status, Boolean(node.tcping_enabled))}</div>
          <div class="muted" style="margin-top:8px;">
            ${isTcpingUnsupported ? 'TCPing 不支持 UDP 探测' : `最近延迟：${escapeHtml(formatLatencyMs(node.tcping_last_latency_ms))}`}
          </div>
          <div class="muted">
            ${isTcpingUnsupported ? '未配置 TCPing 探测端口（TCP）' : `最后采样：${escapeHtml(formatTs(node.tcping_last_sampled_at))}`}
          </div>
        </div>
        <div class="card">
          <h3>可达率</h3>
          <div style="font-size:28px;font-weight:700;">${stats.reachability_rate !== null && stats.reachability_rate !== undefined ? `${Number(stats.reachability_rate).toFixed(2)}%` : '-'}</div>
          <div class="muted" style="margin-top:8px;">${escapeHtml(String(stats.reachable_samples || 0))} / ${escapeHtml(String(stats.total_samples || 0))} 次成功</div>
        </div>
        <div class="card">
          <h3>活动告警</h3>
          <div>${node.tcping_alert_active ? '<span class="pill bad">存在活动告警</span>' : '<span class="pill ok">当前无活动告警</span>'}</div>
          <div class="muted" style="margin-top:8px;">${node.tcping_active_alert?.latest_error ? escapeHtml(node.tcping_active_alert.latest_error) : '暂无告警详情'}</div>
        </div>
      </div>

      <div class="card" style="margin-top:12px;">
        <h3>延迟变化曲线</h3>
        ${isTcpingUnsupported
          ? '<div class="notice">TCPing 不支持 UDP 探测。请在节点高级设置中配置 TCPing 探测端口（TCP）。</div>'
          : html`
              <div class="muted">红底点表示不可连通。线条颜色表示探针归属地区，可在下面筛选显示。</div>
              ${tcpingSeries.length > 1 ? html`
                <div class="row" id="tcpingSeriesFilters" style="flex-wrap: wrap; gap: 10px; margin-top: 10px;">
                  ${tcpingSeries.map((s) => `
                    <label class="row" style="align-items:center; gap:6px;">
                      <input type="checkbox" data-series-key="${escapeHtml(String(s.key || ''))}" checked>
                      <span style="width:10px;height:10px;border-radius:999px;background:${escapeHtml(String(s.color || 'rgba(15,118,110,0.95)'))};display:inline-block;"></span>
                      <span class="muted">${escapeHtml(String(s.label || '-'))}</span>
                    </label>
                  `).join('')}
                </div>
              ` : ''}
              <div style="margin-top:10px;" id="tcpingChartBox">${buildTcpingChartBySeries(tcpingSeries)}</div>
            `}
      </div>

      <div class="card" style="margin-top:12px;">
        <h3>告警记录</h3>
        ${isTcpingUnsupported ? '<div class="notice" style="margin-top:10px;">该节点当前未开启 TCPing 探测，暂无告警。</div>' : (alerts.length ? html`
          <table class="table" style="margin-top:10px;">
            <thead>
              <tr><th>ID</th><th>状态</th><th>开始时间</th><th>触发时间</th><th>恢复时间</th><th>错误</th></tr>
            </thead>
            <tbody>
              ${alerts.map((alertItem) => `
                <tr>
                  <td>${alertItem.id}</td>
                  <td>${alertItem.status === 'active' ? '<span class="pill bad">活动中</span>' : '<span class="pill ok">已恢复</span>'}</td>
                  <td class="muted">${escapeHtml(formatTs(alertItem.started_at))}</td>
                  <td class="muted">${escapeHtml(formatTs(alertItem.triggered_at))}</td>
                  <td class="muted">${escapeHtml(formatTs(alertItem.recovered_at))}</td>
                  <td class="muted">${escapeHtml(alertItem.latest_error || '-')}</td>
                </tr>
              `).join('')}
            </tbody>
          </table>
        ` : '<div class="notice" style="margin-top:10px;">当前没有告警记录。</div>')}
      </div>
    </div>
  `);

  if (!isTcpingUnsupported && tcpingSeries.length > 1) {
    const filterWrap = qs('#tcpingSeriesFilters');
    const chartBox = qs('#tcpingChartBox');
    if (filterWrap && chartBox) {
      const syncChart = () => {
        const selectedKeys = qsa('input[data-series-key]', filterWrap)
          .filter((input) => input.checked)
          .map((input) => String(input.getAttribute('data-series-key') || ''))
          .filter(Boolean);
        chartBox.innerHTML = buildTcpingChartBySeries(tcpingSeries, selectedKeys);
      };
      qsa('input[data-series-key]', filterWrap).forEach((input) => {
        input.addEventListener('change', syncChart);
      });
    }
  }
}

function renderNodeAdvancedSettings(prefix, values = {}, isSuperAdmin = false) {
  return html`
    <details class="settings-collapse" style="grid-column: 1 / -1;">
      <summary>高级设置（限速 / 设备 / TCPing 参数）</summary>
      <div class="grid cols-2" style="margin-top:10px;">
        <div class="field"><label>套餐扣费倍率</label><input id="${prefix}_traffic_multiplier" type="number" min="0.1" max="100" step="0.1" value="${escapeHtml(String(Number(values?.traffic_multiplier || 1) || 1))}"></div>
        <div class="field"><label>同一 IP 跨节点限制</label><input id="${prefix}_cross_node_ip_limit" type="number" min="0" value="${escapeHtml(String(Number(values?.cross_node_ip_limit || 0) || 0))}"></div>
        <div class="field"><label>节点设备限制(0=不限)</label><input id="${prefix}_device_limit" type="number" min="0" value="${escapeHtml(String(Number(values?.device_limit || 0) || 0))}"></div>
        <div class="field"><label>节点连接数限制(0=不限)</label><input id="${prefix}_connection_limit" type="number" min="0" value="${escapeHtml(String(Number(values?.connection_limit || 0) || 0))}"></div>
        <div class="field"><label>上传限速(Mbps, 0=不限)</label><input id="${prefix}_speed_up" type="number" min="0" value="${escapeHtml(String(Number(values?.speed_limit_up || 0) || 0))}"></div>
        <div class="field"><label>下载限速(Mbps, 0=不限)</label><input id="${prefix}_speed_down" type="number" min="0" value="${escapeHtml(String(Number(values?.speed_limit_down || 0) || 0))}"></div>
        <div class="field"><label>同用户多 IP 并发上限(仅超管)</label><input id="${prefix}_concurrent_ip" type="number" min="0" value="${escapeHtml(String(Number(values?.concurrent_ip_limit || 0) || 0))}" ${isSuperAdmin ? '' : 'disabled'}></div>
        <div class="field"><label>TCPing 探测主机</label><input id="${prefix}_tcping_host" value="${escapeHtml(String(values?.tcping_host || ''))}" placeholder="留空则跟随节点域名/IP"></div>
        <div class="field"><label>TCPing 探测端口</label><input id="${prefix}_tcping_port" type="number" min="1" max="65535" value="${escapeHtml(values?.tcping_port ? String(values.tcping_port) : '')}" placeholder="留空则跟随访问端口（UDP 协议留空则关闭探测）"></div>
        <div class="field"><label>TCPing 间隔(秒)</label><input id="${prefix}_tcping_interval" type="number" min="15" max="3600" value="${escapeHtml(String(Number(values?.tcping_interval_seconds || 60) || 60))}"></div>
        <div class="field"><label>TCPing 超时(ms)</label><input id="${prefix}_tcping_timeout" type="number" min="500" max="60000" value="${escapeHtml(String(Number(values?.tcping_timeout_ms || 3000) || 3000))}"></div>
        <div class="field"><label>连续不可达多久告警(秒)</label><input id="${prefix}_tcping_alert_after" type="number" min="60" max="86400" value="${escapeHtml(String(Number(values?.tcping_alert_after_seconds || 300) || 300))}"></div>
        <div class="field"><label>恢复稳定多久解除告警(秒)</label><input id="${prefix}_tcping_recover_after" type="number" min="30" max="86400" value="${escapeHtml(String(Number(values?.tcping_recover_after_seconds || 120) || 120))}"></div>
      </div>
      <div class="muted" style="margin-top:10px;">
        TCPing 由独立 Agent 探测，系统统一开启，不支持单独关闭。UDP 协议节点若未配置探测端口，将显示“不支持 UDP 探测”。Agent 掉线不会影响节点转发，只会影响监控数据。
      </div>
    </details>
  `;
}

async function renderNewNode(errorText = '') {
  if (!requireAuth()) return;
  const me = await loadMe();
  const protocolOptions = await loadNodeProtocolOptions();
  const defaultProtocol = protocolOptions[0]?.value || 'vmess';
  const defaultPort = generateRandomNodePort(50000, 65535);
  const initialSettings = mergeProtocolSettingsForEditor(defaultProtocol);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>新增节点</h2>
        <a class="btn" href="#/nodes">返回</a>
      </div>
      <div id="n_form_error">${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}</div>
      <div class="grid cols-2" style="margin-top: 10px;">
        <div class="field"><label>节点名称</label><input id="n_name" placeholder="例如：香港 01"></div>
        <div class="field"><label>协议</label>
          <select id="n_protocol">
            ${buildProtocolOptionHtml(protocolOptions, defaultProtocol)}
          </select>
        </div>
        <div class="field">
          <label>落地国家 / 地区</label>
          <input id="n_location" list="nodeLocationOptions" placeholder="必选，可搜索，例如 HK | Hong Kong 香港" required>
        </div>
        <div class="field">
          <label>最低用户等级</label>
          <select id="n_min_trust">
            ${buildTrustLevelOptionHtml(0, false)}
          </select>
        </div>
        <div class="field" style="grid-column: 1 / -1;">
          <div class="row" style="justify-content: space-between; align-items: center;">
            <label style="margin:0;">协议参数（可视化）</label>
            <button class="btn small" id="n_reset_settings_btn" type="button">重置当前协议参数</button>
          </div>
          <div id="n_settings_form" class="grid cols-2 protocol-settings-grid"></div>
          <div class="muted">切换协议后，参数表单会自动切换成对应配置，不需要手填 JSON。</div>
        </div>
        <div class="field"><label>节点域名 / IP</label><input id="n_host" placeholder="例如 example.com 或 1.2.3.4"></div>
        <div class="field"><label>用户访问端口</label><input id="n_port" type="number" value="${defaultPort}"></div>
        <div class="field"><label>服务端口</label><input id="n_service_port" type="number" value="${defaultPort}" placeholder="例如 8443"></div>
        <div class="field">
          <label>端口联动</label>
          <label class="row" style="align-items:center; gap:8px;">
            <input id="n_port_link" type="checkbox" checked>
            <span class="muted">开启时：服务端口跟随访问端口</span>
          </label>
        </div>
        <div class="field"><label>节点总流量上限(GB, 0=不限)</label><input id="n_traffic_limit_gb" type="number" step="0.01" value="0"></div>
        ${renderNodeAdvancedSettings('n', {}, Boolean(me?.is_super_admin))}
      </div>
      ${renderNodeLocationDatalist()}
      <div class="row end" style="margin-top: 12px;">
        <button class="btn primary" id="createNodeBtn">创建</button>
      </div>
      <div class="muted" style="margin-top: 10px;">
        创建后可在“分享/权限”里添加指定用户，或按用户等级开放。VLESS 已内置 TLS / REALITY 两档切换。
      </div>
    </div>
  `);

  const nProtocol = qs('#n_protocol');
  const nSettingsForm = qs('#n_settings_form');
  const nResetSettingsBtn = qs('#n_reset_settings_btn');
  const nPortInput = qs('#n_port');
  const nServicePortInput = qs('#n_service_port');
  const nPortLinkInput = qs('#n_port_link');
  const nFormError = qs('#n_form_error');
  const newNodeFieldMap = {
    name: '#n_name',
    protocol: '#n_protocol',
    location_code: '#n_location',
    location_name: '#n_location',
    host: '#n_host',
    port: '#n_port',
    service_port: '#n_service_port',
    settings: '#n_settings_form',
    traffic_limit: '#n_traffic_limit_gb',
    traffic_multiplier: '#n_traffic_multiplier',
    device_limit: '#n_device_limit',
    connection_limit: '#n_connection_limit',
    speed_limit_up: '#n_speed_up',
    speed_limit_down: '#n_speed_down',
    cross_node_ip_limit: '#n_cross_node_ip_limit',
    concurrent_ip_limit: '#n_concurrent_ip',
    tcping_host: '#n_tcping_host',
    tcping_port: '#n_tcping_port',
    tcping_interval_seconds: '#n_tcping_interval',
    tcping_timeout_ms: '#n_tcping_timeout',
    tcping_alert_after_seconds: '#n_tcping_alert_after',
    tcping_recover_after_seconds: '#n_tcping_recover_after',
    access_control: '#n_min_trust',
    'access_control.min_trust_level': '#n_min_trust',
  };
  let servicePortLinked = true;

  const renderNewSettings = (protocol, currentSettings = null) => {
    if (!nSettingsForm) return;
    const merged = mergeProtocolSettingsForEditor(protocol, currentSettings);
    nSettingsForm.innerHTML = renderProtocolSettingsEditor(protocol, merged, {
      host: qs('#n_host')?.value.trim() || '',
    });

    const modeSelect = qs('[data-protocol-mode-select]', nSettingsForm);
    if (modeSelect) {
      modeSelect.addEventListener('change', () => {
        const currentFormSettings = collectProtocolSettingsFromForm('#n_settings_form');
        const nextSettings = applyProtocolEditorModePreset(
          protocol,
          currentFormSettings,
          modeSelect.value,
          { host: qs('#n_host')?.value.trim() || '' }
        );
        renderNewSettings(protocol, nextSettings);
      });
    }
  };

  renderNewSettings(defaultProtocol, initialSettings);

  if (nProtocol) {
    nProtocol.addEventListener('change', () => {
      renderNewSettings(nProtocol.value, null);
    });
  }
  if (nResetSettingsBtn) {
    nResetSettingsBtn.addEventListener('click', () => {
      if (!confirm('确认重置当前协议参数为推荐值？')) return;
      renderNewSettings(nProtocol?.value || defaultProtocol, null);
    });
  }

  const applyNewPortLinkState = () => {
    servicePortLinked = Boolean(nPortLinkInput?.checked);
    if (nServicePortInput) {
      nServicePortInput.readOnly = servicePortLinked;
    }
    if (servicePortLinked && nPortInput && nServicePortInput) {
      nServicePortInput.value = nPortInput.value;
    }
  };

  if (nPortLinkInput) {
    nPortLinkInput.addEventListener('change', applyNewPortLinkState);
  }

  if (nPortInput && nServicePortInput) {
    nPortInput.addEventListener('input', () => {
      if (!servicePortLinked) return;
      nServicePortInput.value = nPortInput.value;
    });

    nServicePortInput.addEventListener('input', () => {
      if (servicePortLinked) return;
      const accessPort = String(nPortInput.value || '').trim();
      const servicePort = String(nServicePortInput.value || '').trim();
      // Empty service port means fallback to access port on backend, so keep linked state.
      servicePortLinked = (servicePort === '' || servicePort === accessPort);
      if (nPortLinkInput) nPortLinkInput.checked = servicePortLinked;
    });
  }

  const nHostInput = qs('#n_host');
  if (nHostInput) {
    nHostInput.addEventListener('change', () => {
      const protocol = nProtocol?.value || defaultProtocol;
      let currentSettings = collectProtocolSettingsFromForm('#n_settings_form');
      const mode = qs('[data-protocol-mode-select]', nSettingsForm)?.value || '';
      if (mode) {
        currentSettings = applyProtocolEditorModePreset(protocol, currentSettings, mode, {
          host: nHostInput.value.trim() || '',
        });
      }
      renderNewSettings(protocol, currentSettings);
    });
  }

  applyNewPortLinkState();
  bindFormValidationAutoClear(VIEW, nFormError);

  qs('#createNodeBtn').addEventListener('click', async () => {
    clearFormValidationState(VIEW);
    setInlineFormError(nFormError, '');

    try {
      const trafficLimitGb = Number(qs('#n_traffic_limit_gb').value) || 0;
      const trafficLimitKb = trafficLimitGb > 0 ? Math.floor(trafficLimitGb * 1024 * 1024) : 0;
      const settings = collectProtocolSettingsFromForm('#n_settings_form');
      const location = readNodeLocationSelection('n_location');
      const accessPort = Number(qs('#n_port').value);
      const isPortLinked = Boolean(qs('#n_port_link')?.checked);
      const rawServicePort = String(qs('#n_service_port')?.value || '').trim();
      const parsedServicePort = rawServicePort ? Number(rawServicePort) : null;
      const rawTcpingPort = String(qs('#n_tcping_port')?.value || '').trim();
      const parsedTcpingPort = rawTcpingPort ? Number(rawTcpingPort) : null;
      const payload = {
        name: qs('#n_name').value.trim(),
        protocol: qs('#n_protocol').value,
        location_code: location.location_code,
        location_name: location.location_name,
        host: qs('#n_host').value.trim(),
        port: accessPort,
        service_port: isPortLinked
          ? accessPort
          : ((parsedServicePort && Number.isFinite(parsedServicePort) && parsedServicePort > 0)
            ? parsedServicePort
            : null),
        settings,
        traffic_limit: trafficLimitKb,
        traffic_multiplier: Number(qs('#n_traffic_multiplier').value) || 1,
        device_limit: Number(qs('#n_device_limit').value) || 0,
        connection_limit: Number(qs('#n_connection_limit').value) || 0,
        speed_limit_up: Number(qs('#n_speed_up').value) || 0,
        speed_limit_down: Number(qs('#n_speed_down').value) || 0,
        cross_node_ip_limit: Number(qs('#n_cross_node_ip_limit').value) || 0,
        tcping_host: String(qs('#n_tcping_host')?.value || '').trim() || null,
        tcping_port: (parsedTcpingPort && Number.isFinite(parsedTcpingPort) && parsedTcpingPort > 0) ? parsedTcpingPort : null,
        tcping_interval_seconds: Number(qs('#n_tcping_interval').value) || 60,
        tcping_timeout_ms: Number(qs('#n_tcping_timeout').value) || 3000,
        tcping_alert_after_seconds: Number(qs('#n_tcping_alert_after').value) || 300,
        tcping_recover_after_seconds: Number(qs('#n_tcping_recover_after').value) || 120,
        access_control: { min_trust_level: Number(qs('#n_min_trust').value) }
      };
      if (me?.is_super_admin) payload.concurrent_ip_limit = Number(qs('#n_concurrent_ip').value) || 0;

      await apiFetch('/api/v1/user/server-nodes', { method: 'POST', body: payload });
      location.hash = '#/nodes';
    } catch (e) {
      presentFormSubmitError({
        rootNode: VIEW,
        errorBox: nFormError,
        fieldMap: newNodeFieldMap,
        error: e,
        fallbackMessage: e.message || '创建失败',
      });
    }
  });
}

async function renderEditNode(nodeId, errorText = '') {
  if (!requireAuth()) return;
  const me = await loadMe();
  const protocolOptions = await loadNodeProtocolOptions();
  const protocolMap = Object.fromEntries(protocolOptions.map((item) => [item.value, item.label]));

  let node = null;
  try {
    const res = await apiFetch(`/api/v1/user/server-nodes/${nodeId}`);
    node = res?.data || null;
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '节点不存在')}</div>`);
    return;
  }

  const trafficLimitGb = node?.traffic_limit ? (Number(node.traffic_limit) / 1024 / 1024) : 0;

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>编辑节点：${escapeHtml(node?.name || nodeId)}</h2>
        <a class="btn" href="#/nodes">返回</a>
      </div>
      <div id="en_form_error">${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}</div>
      <div class="grid cols-2" style="margin-top: 10px;">
        <div class="field"><label>节点名称</label><input id="en_name" value="${escapeHtml(node?.name || '')}"></div>
        <div class="field"><label>在线状态（V2bX 回传）</label><div>${pillStatus(node)}</div></div>
        <div class="field">
          <label>落地国家 / 地区</label>
          <input id="en_location" list="nodeLocationOptions" value="${escapeHtml(getNodeLocationInputValue(node?.location_code, node?.location_name))}" placeholder="必选，可搜索，例如 HK | Hong Kong 香港" required>
        </div>
        <div class="field">
          <label>最低用户等级</label>
          <select id="en_min_trust">
            ${buildTrustLevelOptionHtml(node?.access_control?.min_trust_level ?? 0, false)}
          </select>
        </div>
        <div class="field"><label>节点域名 / IP</label><input id="en_host" value="${escapeHtml(node?.host || '')}"></div>
        <div class="field"><label>用户访问端口</label><input id="en_port" type="number" value="${node?.port || 443}"></div>
        <div class="field"><label>服务端口</label><input id="en_service_port" type="number" value="${node?.service_port ?? ''}" placeholder="例如 8443"></div>
        <div class="field">
          <label>端口联动</label>
          <label class="row" style="align-items:center; gap:8px;">
            <input id="en_port_link" type="checkbox" ${((node?.service_port ?? null) === null || Number(node?.service_port) === Number(node?.port)) ? 'checked' : ''}>
            <span class="muted">开启时：服务端口跟随访问端口</span>
          </label>
        </div>
        <div class="field"><label>协议</label><input value="${escapeHtml(protocolDisplayName(node?.protocol, protocolMap))} (${escapeHtml(node?.protocol || '')})" disabled></div>
        <div class="field" style="grid-column: 1 / -1;">
          <div class="row" style="justify-content: space-between; align-items: center;">
            <label style="margin:0;">协议参数（可视化）</label>
            <button class="btn small" id="en_reset_settings_btn" type="button">恢复推荐参数</button>
          </div>
          <div id="en_settings_form" class="grid cols-2 protocol-settings-grid"></div>
          <div class="muted">可直接可视化修改参数，不需要手填 JSON。</div>
        </div>
        <div class="field"><label>节点总流量上限(GB, 0=不限)</label><input id="en_traffic_limit_gb" type="number" step="0.01" value="${trafficLimitGb || 0}"></div>
        ${renderNodeAdvancedSettings('en', node || {}, Boolean(me?.is_super_admin))}
      </div>
      ${renderNodeLocationDatalist()}
      <div class="row end" style="margin-top: 12px;">
        <button class="btn primary" id="en_save">保存</button>
      </div>
      <div class="muted" style="margin-top: 10px;">节点总流量达到上限后，将停止向该节点下发用户，V2bX 下发会自动清空该节点用户列表。</div>
    </div>
  `);

  const enSettingsForm = qs('#en_settings_form');
  const renderEditSettings = (currentSettings = null) => {
    if (!enSettingsForm) return;
    const merged = mergeProtocolSettingsForEditor(node?.protocol, currentSettings);
    enSettingsForm.innerHTML = renderProtocolSettingsEditor(node?.protocol, merged, {
      host: qs('#en_host')?.value.trim() || node?.host || '',
    });

    const modeSelect = qs('[data-protocol-mode-select]', enSettingsForm);
    if (modeSelect) {
      modeSelect.addEventListener('change', () => {
        const currentFormSettings = collectProtocolSettingsFromForm('#en_settings_form');
        const nextSettings = applyProtocolEditorModePreset(
          node?.protocol,
          currentFormSettings,
          modeSelect.value,
          { host: qs('#en_host')?.value.trim() || node?.host || '' }
        );
        renderEditSettings(nextSettings);
      });
    }
  };

  renderEditSettings(node?.settings || null);

  const enResetSettingsBtn = qs('#en_reset_settings_btn');
  if (enResetSettingsBtn) {
    enResetSettingsBtn.addEventListener('click', () => {
      if (!confirm('确认恢复推荐参数？')) return;
      renderEditSettings(null);
    });
  }

  const enPortInput = qs('#en_port');
  const enServicePortInput = qs('#en_service_port');
  const enPortLinkInput = qs('#en_port_link');
  const enFormError = qs('#en_form_error');
  const editNodeFieldMap = {
    name: '#en_name',
    location_code: '#en_location',
    location_name: '#en_location',
    host: '#en_host',
    port: '#en_port',
    service_port: '#en_service_port',
    settings: '#en_settings_form',
    traffic_limit: '#en_traffic_limit_gb',
    traffic_multiplier: '#en_traffic_multiplier',
    device_limit: '#en_device_limit',
    connection_limit: '#en_connection_limit',
    speed_limit_up: '#en_speed_up',
    speed_limit_down: '#en_speed_down',
    cross_node_ip_limit: '#en_cross_node_ip_limit',
    concurrent_ip_limit: '#en_concurrent_ip',
    tcping_host: '#en_tcping_host',
    tcping_port: '#en_tcping_port',
    tcping_interval_seconds: '#en_tcping_interval',
    tcping_timeout_ms: '#en_tcping_timeout',
    tcping_alert_after_seconds: '#en_tcping_alert_after',
    tcping_recover_after_seconds: '#en_tcping_recover_after',
    access_control: '#en_min_trust',
    'access_control.min_trust_level': '#en_min_trust',
  };
  let editPortLinked = Boolean(enPortLinkInput?.checked);

  const applyEditPortLinkState = () => {
    editPortLinked = Boolean(enPortLinkInput?.checked);
    if (enServicePortInput) {
      enServicePortInput.readOnly = editPortLinked;
    }
    if (editPortLinked && enPortInput && enServicePortInput) {
      enServicePortInput.value = enPortInput.value;
    }
  };

  if (enPortLinkInput) {
    enPortLinkInput.addEventListener('change', applyEditPortLinkState);
  }

  if (enPortInput && enServicePortInput) {
    enPortInput.addEventListener('input', () => {
      if (!editPortLinked) return;
      enServicePortInput.value = enPortInput.value;
    });

    enServicePortInput.addEventListener('input', () => {
      if (editPortLinked) return;
      const accessPort = String(enPortInput.value || '').trim();
      const servicePort = String(enServicePortInput.value || '').trim();
      editPortLinked = (servicePort === '' || servicePort === accessPort);
      if (enPortLinkInput) enPortLinkInput.checked = editPortLinked;
    });
  }

  const enHostInput = qs('#en_host');
  if (enHostInput) {
    enHostInput.addEventListener('change', () => {
      let currentSettings = collectProtocolSettingsFromForm('#en_settings_form');
      const mode = qs('[data-protocol-mode-select]', enSettingsForm)?.value || '';
      if (mode) {
        currentSettings = applyProtocolEditorModePreset(node?.protocol, currentSettings, mode, {
          host: enHostInput.value.trim() || node?.host || '',
        });
      }
      renderEditSettings(currentSettings);
    });
  }

  applyEditPortLinkState();
  bindFormValidationAutoClear(VIEW, enFormError);

  qs('#en_save').addEventListener('click', async () => {
    clearFormValidationState(VIEW);
    setInlineFormError(enFormError, '');

    try {
      const limitGb = Number(qs('#en_traffic_limit_gb').value) || 0;
      const limitKb = limitGb > 0 ? Math.floor(limitGb * 1024 * 1024) : 0;
      const settings = collectProtocolSettingsFromForm('#en_settings_form');
      const location = readNodeLocationSelection('en_location');
      const accessPort = Number(qs('#en_port').value);
      const isPortLinked = Boolean(qs('#en_port_link')?.checked);
      const rawServicePort = String(qs('#en_service_port')?.value || '').trim();
      const parsedServicePort = rawServicePort ? Number(rawServicePort) : null;
      const rawTcpingPort = String(qs('#en_tcping_port')?.value || '').trim();
      const parsedTcpingPort = rawTcpingPort ? Number(rawTcpingPort) : null;
      const payload = {
        name: qs('#en_name').value.trim(),
        location_code: location.location_code,
        location_name: location.location_name,
        host: qs('#en_host').value.trim(),
        port: accessPort,
        service_port: isPortLinked
          ? accessPort
          : ((parsedServicePort && Number.isFinite(parsedServicePort) && parsedServicePort > 0)
            ? parsedServicePort
            : null),
        settings,
        traffic_limit: limitKb,
        traffic_multiplier: Number(qs('#en_traffic_multiplier').value) || 1,
        device_limit: Number(qs('#en_device_limit').value) || 0,
        connection_limit: Number(qs('#en_connection_limit').value) || 0,
        speed_limit_up: Number(qs('#en_speed_up').value) || 0,
        speed_limit_down: Number(qs('#en_speed_down').value) || 0,
        cross_node_ip_limit: Number(qs('#en_cross_node_ip_limit').value) || 0,
        tcping_host: String(qs('#en_tcping_host')?.value || '').trim() || null,
        tcping_port: (parsedTcpingPort && Number.isFinite(parsedTcpingPort) && parsedTcpingPort > 0) ? parsedTcpingPort : null,
        tcping_interval_seconds: Number(qs('#en_tcping_interval').value) || 60,
        tcping_timeout_ms: Number(qs('#en_tcping_timeout').value) || 3000,
        tcping_alert_after_seconds: Number(qs('#en_tcping_alert_after').value) || 300,
        tcping_recover_after_seconds: Number(qs('#en_tcping_recover_after').value) || 120,
        access_control: {
          ...(isPlainObject(node?.access_control) ? deepClone(node.access_control) : {}),
          min_trust_level: Number(qs('#en_min_trust').value) || 0,
        },
      };
      if (me?.is_super_admin) payload.concurrent_ip_limit = Number(qs('#en_concurrent_ip').value) || 0;
      await apiFetch(`/api/v1/user/server-nodes/${nodeId}`, { method: 'PUT', body: payload });
      location.hash = '#/nodes';
    } catch (e) {
      presentFormSubmitError({
        rootNode: VIEW,
        errorBox: enFormError,
        fieldMap: editNodeFieldMap,
        error: e,
        fallbackMessage: e.message || '保存失败',
      });
    }
  });
}

async function renderNodeAccess(nodeId, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const nodeRes = await apiFetch(`/api/v1/user/server-nodes/${nodeId}`);
  const node = nodeRes?.data || null;
  const freeQuota = node?.access_control?.free_quota_gb_by_trust_level || {};
  const q = (lvl) => {
    const v = freeQuota[String(lvl)] ?? freeQuota[lvl] ?? 0;
    return Number(v) || 0;
  };

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>分享/权限：${escapeHtml(node?.name || nodeId)}</h2>
        <a class="btn" href="#/nodes">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid cols-2" style="margin-top: 10px;">
        <div class="field">
          <label>最低用户等级（0-4）</label>
          <select id="minTrust">
            ${buildTrustLevelOptionHtml(node?.access_control?.min_trust_level ?? 0, false)}
          </select>
        </div>
        <div class="field">
          <label>指定可用用户 ID（多个用逗号）</label>
          <input id="userIds" placeholder="例如 1,2,3" value="${asArray(node?.access_control?.authorized_users).join(',')}">
        </div>
      </div>

      <div class="card" style="margin-top: 12px;">
        <h3>免费额度（按用户等级，GB/月）</h3>
        <div class="muted">为不同等级用户提供该节点的免费使用额度，超出后不再下发到该节点。</div>
        <div class="grid cols-2" style="margin-top: 10px;">
          <div class="field"><label>TL0</label><input id="fq0" type="number" min="0" step="0.01" value="${q(0)}"></div>
          <div class="field"><label>TL1</label><input id="fq1" type="number" min="0" step="0.01" value="${q(1)}"></div>
          <div class="field"><label>TL2</label><input id="fq2" type="number" min="0" step="0.01" value="${q(2)}"></div>
          <div class="field"><label>TL3</label><input id="fq3" type="number" min="0" step="0.01" value="${q(3)}"></div>
          <div class="field"><label>TL4</label><input id="fq4" type="number" min="0" step="0.01" value="${q(4)}"></div>
        </div>
      </div>
      <div class="row end" style="margin-top: 12px;">
        <button class="btn primary" id="saveAccessBtn">保存</button>
      </div>
    </div>
  `);

  qs('#saveAccessBtn').addEventListener('click', async () => {
    try {
      const minTrustLevel = Number(qs('#minTrust').value);
      const ids = qs('#userIds').value
        .split(',')
        .map(s => s.trim())
        .filter(Boolean)
        .map(v => Number(v))
        .filter(v => Number.isFinite(v) && v > 0);

      const free_quota_gb_by_trust_level = {
        0: Number(qs('#fq0').value) || 0,
        1: Number(qs('#fq1').value) || 0,
        2: Number(qs('#fq2').value) || 0,
        3: Number(qs('#fq3').value) || 0,
        4: Number(qs('#fq4').value) || 0,
      };

      await apiFetch(`/api/v1/user/server-nodes/${nodeId}/access`, {
        method: 'POST',
        body: {
          min_trust_level: Number.isFinite(minTrustLevel) ? minTrustLevel : null,
          authorized_users: ids,
          free_quota_gb_by_trust_level,
        }
      });
      location.hash = '#/nodes';
    } catch (e) {
      renderNodeAccess(nodeId, e.message || '保存失败');
    }
  });
}

async function renderNodeAuditRules(nodeId, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const list = await apiFetch(`/api/v1/user/server-nodes/${nodeId}/audit-rules`);
  const rules = asArray(list?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>审计规则：节点 ${nodeId}</h2>
        <a class="btn" href="#/nodes">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="row" style="margin-top: 10px;">
        <div class="field" style="min-width: 140px;">
          <label>类型</label>
          <select id="r_type">
            <option value="domain">域名</option>
            <option value="protocol">协议特征</option>
            <option value="ip">IP / 网段</option>
          </select>
        </div>
        <div class="field" style="flex: 1;">
          <label>匹配内容</label>
          <input id="r_pattern" placeholder="example.com / tcp / 1.2.3.* / 1.2.3.0/24">
        </div>
        <div class="field" style="min-width: 140px;">
          <label>处理方式</label>
          <select id="r_action">
            <option value="block">拦截</option>
            <option value="allow">放行</option>
            <option value="log">仅记录</option>
          </select>
        </div>
        <div class="field" style="min-width: 120px;">
          <label>启用</label>
          <select id="r_active">
            <option value="1">是</option>
            <option value="0">否</option>
          </select>
        </div>
        <div class="field" style="align-self: end;">
          <button class="btn primary" id="addRuleBtn">新增</button>
        </div>
      </div>

      <table class="table" style="margin-top: 12px;">
        <thead>
          <tr><th>ID</th><th>类型</th><th>匹配内容</th><th>处理方式</th><th>启用</th><th></th></tr>
        </thead>
        <tbody>
          ${rules.map(r => html`
            <tr>
              <td>${r.id}</td>
              <td><span class="pill">${r.rule_type === 'domain' ? '域名' : (r.rule_type === 'protocol' ? '协议特征' : (r.rule_type === 'ip' ? 'IP / 网段' : r.rule_type))}</span></td>
              <td class="muted">${escapeHtml(r.rule_pattern)}</td>
              <td><span class="pill">${r.action === 'block' ? '拦截' : (r.action === 'allow' ? '放行' : (r.action === 'log' ? '仅记录' : r.action))}</span></td>
              <td>${r.is_active ? `<span class="pill ok">启用</span>` : `<span class="pill">停用</span>`}</td>
              <td class="row end">
                <button class="btn small danger" data-del="${r.id}">删除</button>
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);

  qs('#addRuleBtn').addEventListener('click', async () => {
    try {
      await apiFetch(`/api/v1/user/server-nodes/${nodeId}/audit-rules`, {
        method: 'POST',
        body: {
          rule_type: qs('#r_type').value,
          rule_pattern: qs('#r_pattern').value.trim(),
          action: qs('#r_action').value,
          is_active: qs('#r_active').value === '1'
        }
      });
      await renderNodeAuditRules(nodeId);
    } catch (e) {
      renderNodeAuditRules(nodeId, e.message || '新增失败');
    }
  });

  qsa('button[data-del]').forEach(btn => {
    btn.addEventListener('click', async () => {
      if (!confirm('确认删除该规则？')) return;
      const ruleId = btn.getAttribute('data-del');
      await apiFetch(`/api/v1/user/server-nodes/${nodeId}/audit-rules/${ruleId}`, { method: 'DELETE' });
      await renderNodeAuditRules(nodeId);
    });
  });
}

async function renderPlans(purchaseToken = '') {
  if (!requireAuth()) return;
  await loadMe();

  const token = String(purchaseToken || '').trim();
  const planEndpoint = token
    ? `/api/v1/user/plan/fetch?token=${encodeURIComponent(token)}`
    : '/api/v1/user/plan/fetch';
  const res = await apiFetch(planEndpoint);
  const plans = token ? [res?.data].filter(Boolean) : asArray(res?.data);
  const getPeriodOptions = (plan) => PLAN_PRICE_FIELDS
    .map((item) => {
      const raw = plan?.[item.key];
      if (raw === null || raw === undefined || raw === '') {
        return { ...item, price: null };
      }
      const parsed = Number(raw);
      return { ...item, price: Number.isFinite(parsed) ? parsed : null };
    })
    .filter(item => item.price !== null && item.price >= 0);
  const formatCny = (fen) => `¥${(Number(fen) / 100).toFixed(2)}`;

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>${token ? '专属套餐购买' : '可购买套餐'}</h2>
        <div class="muted">${token ? '你正在通过专属链接购买套餐' : '含 legacy / 节点套餐'}</div>
      </div>
      ${token ? '<div class="notice" style="margin-top:10px;">该页面来自专属链接，套餐不会在公开列表展示。</div>' : ''}
      <table class="table" style="margin-top: 10px;">
        <thead>
          <tr><th>ID</th><th>名称</th><th>发布者</th><th>类型</th><th>描述</th><th>购买周期</th><th></th></tr>
        </thead>
        <tbody>
          ${plans.map(p => {
            const options = getPeriodOptions(p);
            const ownerName = String(p?.owner_display_name || p?.owner?.display_name || p?.owner?.linux_do_name || p?.owner?.linux_do_username || p?.owner?.email || '未知');
            return html`
              <tr>
                <td>${p.id}</td>
                <td>${escapeHtml(String(p.name || ''))}</td>
                <td class="muted">${escapeHtml(ownerName)}</td>
                <td><span class="pill">${escapeHtml(String(p.scope || 'legacy'))}</span></td>
                <td class="muted">${escapeHtml(String(p.content || '').slice(0, 120))}</td>
                <td>
                  <select data-period-for="${p.id}" ${options.length ? '' : 'disabled'}>
                    ${options.length ? options.map(opt => `<option value="${opt.key}">${opt.label} (${formatCny(opt.price)})</option>`).join('') : '<option value="">暂无可购买周期</option>'}
                  </select>
                </td>
                <td class="row end">
                  <button class="btn small primary" data-buy="${p.id}" ${options.length ? '' : 'disabled'}>购买</button>
                </td>
              </tr>
            `;
          }).join('')}
        </tbody>
      </table>
      <div class="muted" style="margin-top: 10px;">购买会创建订单；如是 0 元订单将直接完成。</div>
    </div>
  `);

  qsa('button[data-buy]').forEach(btn => {
    btn.addEventListener('click', async () => {
      const planId = btn.getAttribute('data-buy');
      const periodEl = qs(`select[data-period-for="${planId}"]`);
      const period = periodEl ? periodEl.value : '';
      if (!period) return;
      try {
        const tradeNoRes = await apiFetch('/api/v1/user/order/save', {
          method: 'POST',
          body: { plan_id: Number(planId), period, purchase_token: token || undefined }
        });
        const tradeNo = tradeNoRes?.data;
        location.hash = '#/order/' + tradeNo;
      } catch (e) {
        alert(e.message || '创建订单失败');
      }
    });
  });
}

async function renderOrders() {
  if (!requireAuth()) return;
  await loadMe();

  const res = await apiFetch('/api/v1/user/order/fetch');
  const orders = asArray(res?.data);

  setView(html`
    <div class="card">
      <h2>我的订单</h2>
      <table class="table" style="margin-top: 10px;">
        <thead>
          <tr><th>TradeNo</th><th>套餐</th><th>周期</th><th>金额</th><th>状态</th><th></th></tr>
        </thead>
        <tbody>
          ${orders.map(o => html`
            <tr>
              <td class="muted">${o.trade_no}</td>
              <td>${escapeHtml(o.plan?.name || o.plan_id)}</td>
              <td class="muted">${o.period}</td>
              <td class="muted">${o.total_amount ?? '-'}</td>
              <td><span class="pill">${o.status}</span></td>
              <td class="row end"><a class="btn small" href="#/order/${o.trade_no}">查看</a></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);
}

async function renderOrderDetail(tradeNo, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const disputeEnabled = Boolean(store.me?.refund_dispute_enable);

  let detail = null;
  try {
    const res = await apiFetch('/api/v1/user/order/detail?trade_no=' + encodeURIComponent(tradeNo));
    detail = res?.data || null;
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '订单不存在')}</div>`);
    return;
  }

  const methodsRes = await apiFetch('/api/v1/user/order/getPaymentMethod');
  const methods = asArray(methodsRes?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>订单：${tradeNo}</h2>
        <a class="btn" href="#/orders">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">套餐</div><div class="v">${escapeHtml(detail?.plan?.name || '-')}</div>
        <div class="k">金额(分)</div><div class="v">${detail?.total_amount}</div>
        <div class="k">状态</div><div class="v">${detail?.status}</div>
      </div>

      <div class="card" style="margin-top: 12px;">
        <h3>支付</h3>
        <div class="muted">选择支付方式后将打开支付页面（或返回表单/二维码等）。</div>
        <div class="row" style="margin-top: 10px;">
          <select id="payMethod" style="min-width: 260px;">
            ${methods.map(m => `<option value="${m.id}">${escapeHtml(m.name)}</option>`).join('')}
          </select>
          <button class="btn primary" id="payBtn">发起支付</button>
          <button class="btn danger" id="cancelBtn">取消订单</button>
        </div>
        <div id="payOut" style="margin-top: 10px;"></div>
      </div>

      <div class="card" style="margin-top: 12px;">
        <h3>退款</h3>
        <div class="muted">${disputeEnabled ? '仅支持节点套餐退款；如进入争议将开放投票。' : '仅支持节点套餐退款；争议投票已关闭。'}</div>
        <div class="row" style="margin-top: 10px;">
          <button class="btn" id="refundBtn">申请退款</button>
          <a class="btn" href="#/refunds">查看退款记录</a>
        </div>
      </div>
    </div>
  `);

  qs('#cancelBtn').addEventListener('click', async () => {
    if (!confirm('确认取消该订单？')) return;
    await apiFetch('/api/v1/user/order/cancel', { method: 'POST', body: { trade_no: tradeNo } });
    location.hash = '#/orders';
  });

  qs('#payBtn').addEventListener('click', async () => {
    try {
      const methodId = Number(qs('#payMethod').value);
      const out = await apiFetch('/api/v1/user/order/checkout', {
        method: 'POST',
        body: { trade_no: tradeNo, method: methodId }
      });

      const type = out?.type;
      const data = out?.data;
      const payOut = qs('#payOut');

      if (type === -1) {
        payOut.innerHTML = `<div class="notice ok">0 元订单已完成。</div>`;
        return;
      }

      const paymentUrl = typeof data === 'string' ? normalizeHttpUrl(data) : '';
      if (paymentUrl) {
        const notice = document.createElement('div');
        notice.className = 'notice';
        notice.append(document.createTextNode('支付链接：'));
        const link = document.createElement('a');
        link.href = paymentUrl;
        link.target = '_blank';
        link.rel = 'noopener noreferrer';
        link.textContent = paymentUrl;
        notice.append(link, document.createTextNode(' '));
        const go = document.createElement('button');
        go.className = 'btn small';
        go.type = 'button';
        go.textContent = '跳转';
        go.addEventListener('click', () => { window.location.href = paymentUrl; });
        notice.append(go);
        payOut.replaceChildren(notice);
        return;
      }

      if (typeof data === 'string' && data.includes('<form')) {
        submitSafeEpayPostForm(payOut, data, '收到支付表单，正在跳转...');
        return;
      }

      const notice = document.createElement('div');
      notice.className = 'notice';
      notice.append(document.createTextNode('支付返回：'));
      const pre = document.createElement('pre');
      pre.style.whiteSpace = 'pre-wrap';
      pre.style.color = 'var(--muted)';
      pre.textContent = JSON.stringify(out, null, 2);
      notice.append(pre);
      payOut.replaceChildren(notice);
    } catch (e) {
      renderOrderDetail(tradeNo, e.message || '支付失败');
    }
  });
}

async function renderTickets() {
  if (!requireAuth()) return;
  await loadMe();

  const res = await apiFetch('/api/v1/user/ticket/fetch');
  const tickets = asArray(res?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>我的工单</h2>
        <button class="btn primary" id="newTicketBtn">新建工单</button>
      </div>
      <table class="table" style="margin-top: 10px;">
        <thead>
          <tr><th>ID</th><th>节点</th><th>主题</th><th>状态</th><th></th></tr>
        </thead>
        <tbody>
          ${tickets.map(t => html`
            <tr>
              <td>${t.id}</td>
              <td class="muted">${t.node_id || '-'}</td>
              <td>${escapeHtml(String(t.subject || ''))}</td>
              <td><span class="pill">${t.status}</span></td>
              <td class="row end"><a class="btn small" href="#/ticket/${t.id}">查看</a></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);

  qs('#newTicketBtn').addEventListener('click', () => {
    location.hash = '#/tickets/new';
  });
}

async function renderNewTicket(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const nodesRes = await apiFetch('/api/v1/user/accessible-nodes');
  const nodes = asArray(nodesRes?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>新建工单</h2>
        <a class="btn" href="#/tickets">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid cols-2" style="margin-top: 10px;">
        <div class="field"><label>节点（可选）</label>
          <select id="t_node">
            <option value="">（不指定）</option>
            ${nodes.map(n => `<option value="${escapeHtml(String(n.id))}">${escapeHtml(String(n.name || ''))} (#${escapeHtml(String(n.id))})</option>`).join('')}
          </select>
        </div>
        <div class="field"><label>等级</label>
          <select id="t_level">
            <option value="0">低</option>
            <option value="1">中</option>
            <option value="2">高</option>
          </select>
        </div>
        <div class="field" style="grid-column: 1 / -1;"><label>主题</label><input id="t_subject" placeholder="问题简述"></div>
        <div class="field" style="grid-column: 1 / -1;"><label>内容</label><textarea id="t_message" rows="6" placeholder="详细描述"></textarea></div>
      </div>
      <div class="row end" style="margin-top: 12px;">
        <button class="btn primary" id="t_submit">提交</button>
      </div>
      <div class="muted" style="margin-top: 10px;">如指定节点，将通知该节点负责人（个人管理员）。</div>
    </div>
  `);

  qs('#t_submit').addEventListener('click', async () => {
    try {
      const nodeId = qs('#t_node').value ? Number(qs('#t_node').value) : null;
      await apiFetch('/api/v1/user/ticket/save', {
        method: 'POST',
        body: {
          subject: qs('#t_subject').value.trim(),
          level: qs('#t_level').value,
          message: qs('#t_message').value.trim(),
          node_id: nodeId
        }
      });
      location.hash = '#/tickets';
    } catch (e) {
      renderNewTicket(e.message || '提交失败');
    }
  });
}

async function renderTicketDetail(id, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const res = await apiFetch('/api/v1/user/ticket/fetch?id=' + encodeURIComponent(id));
  const ticket = res?.data || null;

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>工单 #${ticket?.id}</h2>
        <a class="btn" href="#/tickets">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">节点</div><div class="v">${ticket?.node_id || '-'}</div>
        <div class="k">主题</div><div class="v">${escapeHtml(String(ticket?.subject || ''))}</div>
        <div class="k">状态</div><div class="v"><span class="pill">${ticket?.status}</span></div>
      </div>
      <div class="card" style="margin-top: 12px;">
        <h3>消息</h3>
        <div class="grid" style="gap: 8px; margin-top: 10px;">
          ${asArray(ticket?.message).map(m => html`
            <div class="notice ${m.is_me ? 'ok' : ''}">
              <div class="muted">user_id=${m.user_id} · ${m.created_at}</div>
              <div>${escapeHtml(String(m.message || '')).replaceAll('\n','<br>')}</div>
            </div>
          `).join('')}
        </div>
      </div>
      <div class="card" style="margin-top: 12px;">
        <h3>回复</h3>
        <textarea id="replyMsg" rows="4" placeholder="输入回复"></textarea>
        <div class="row end" style="margin-top: 10px;">
          <button class="btn" id="replyBtn">回复</button>
          <button class="btn danger" id="closeBtn">关闭</button>
        </div>
      </div>
    </div>
  `);

  qs('#replyBtn').addEventListener('click', async () => {
    try {
      await apiFetch('/api/v1/user/ticket/reply', { method: 'POST', body: { id: Number(id), message: qs('#replyMsg').value } });
      await renderTicketDetail(id);
    } catch (e) {
      renderTicketDetail(id, e.message || '回复失败');
    }
  });

  qs('#closeBtn').addEventListener('click', async () => {
    if (!confirm('确认关闭工单？')) return;
    await apiFetch('/api/v1/user/ticket/close', { method: 'POST', body: { id: Number(id) } });
    location.hash = '#/tickets';
  });
}

async function renderNodePlans(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const res = await apiFetch('/api/v1/user/node-plans');
  const plans = asArray(res?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>我发布的节点套餐</h2>
        <button class="btn primary" id="newNodePlanBtn">新建</button>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <table class="table" style="margin-top: 10px;">
        <thead>
          <tr><th>ID</th><th>名称</th><th>节点</th><th>可见方式</th><th>试用</th><th>付费额度(GB)</th><th>专属购买链接</th><th></th></tr>
        </thead>
        <tbody>
          ${plans.map(p => html`
            <tr>
              <td>${p.id}</td>
              <td>${escapeHtml(String(p.name || ''))}</td>
              <td class="muted">${asArray(p.node_ids).join(',')}</td>
              <td class="muted">
                ${normalizePlanVisibilityScope(p.visibility_scope) === 'link_only'
                  ? '<span class="pill">仅专属链接</span>'
                  : (normalizePlanVisibilityScope(p.visibility_scope) === 'assigned_only'
                    ? '<span class="pill">仅指定用户</span>'
                    : `<span class="pill ${p.show ? 'ok' : ''}">${p.show ? '公开展示' : '不公开展示'}</span>`)}
              </td>
              <td>${p.allow_trial || hasPositiveTrialQuota(p.free_quota_gb_by_trust_level) ? '<span class="pill ok">允许</span>' : '<span class="pill">不允许</span>'}</td>
              <td class="muted">${p.is_unlimited_traffic ? '<span class="pill ok">无限</span>' : (Number(p.paid_quota_gb ?? p.transfer_enable ?? 0) || 0)}</td>
              <td>
                ${p.share_token
                  ? `<div class="row">
                      <input readonly value="${escapeHtml(buildNodePlanShareLink(p.share_token))}">
                      <button class="btn small" data-copy-plan-link="${escapeHtml(buildNodePlanShareLink(p.share_token))}">复制</button>
                    </div>`
                  : '<span class="muted">未启用</span>'}
              </td>
              <td class="row end">
                <button class="btn small" data-edit="${p.id}">编辑</button>
                <button class="btn small danger" data-del="${p.id}">删除</button>
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
      <div class="muted" style="margin-top: 10px;">普通用户也可以在这里创建并发布自己的节点套餐；套餐只能包含你自己管理的节点。</div>
    </div>
  `);

  qs('#newNodePlanBtn').addEventListener('click', () => location.hash = '#/node-plans/new');
  qsa('button[data-copy-plan-link]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      try {
        await copyText(btn.getAttribute('data-copy-plan-link') || '');
        alert('专属购买链接已复制');
      } catch (e) {
        alert(e.message || '复制失败');
      }
    });
  });
  qsa('button[data-edit]').forEach(b => b.addEventListener('click', () => location.hash = '#/node-plans/edit/' + b.getAttribute('data-edit')));
  qsa('button[data-del]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm('确认删除？')) return;
    try {
      await apiFetch('/api/v1/user/node-plans/' + b.getAttribute('data-del'), { method: 'DELETE' });
      await renderNodePlans();
    } catch (e) {
      await renderNodePlans(e.message || '删除失败');
    }
  }));
}

async function renderNodePlanForm(mode, id = null, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const nodesRes = await apiFetch('/api/v1/user/server-nodes');
  const nodes = asArray(nodesRes?.data);

  let plan = null;
  if (mode === 'edit') {
    const myPlans = await apiFetch('/api/v1/user/node-plans');
    plan = asArray(myPlans?.data).find(p => String(p.id) === String(id)) || null;
    if (!plan) {
      setView(`<div class="notice error">套餐不存在</div>`);
      return;
    }
  }

  const initialPrices = {};
  const initialFreeQuota = normalizeFreeQuotaMap(plan?.free_quota_gb_by_trust_level);
  const initialVisibility = normalizePlanVisibilityScope(plan?.visibility_scope);
  const initialAccessUserIds = asArray(plan?.access_user_ids).join(',');
  const initialShareToken = String(plan?.share_token || '');
  const initialAllowTrial = Boolean(plan?.allow_trial || hasPositiveTrialQuota(plan?.free_quota_gb_by_trust_level));
  const initialUnlimitedTraffic = Boolean(plan?.is_unlimited_traffic);
  if (plan) {
    PLAN_PRICE_FIELDS.forEach(({ key: k }) => {
      if (plan[k] !== null && plan[k] !== undefined) initialPrices[k] = plan[k];
    });
  } else {
    initialPrices.month_price = 0;
  }

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>${mode === 'new' ? '新建节点套餐' : '编辑节点套餐 #' + id}</h2>
        <a class="btn" href="#/node-plans">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid cols-2" style="margin-top: 10px;">
        <div class="field" style="grid-column: 1 / -1;"><label>名称</label><input id="np_name" value="${escapeHtml(String(plan?.name || ''))}"></div>
        <div class="field">
          <label>可见方式</label>
          <select id="np_visibility_scope">
            <option value="public">公开展示</option>
            <option value="link_only">仅专属链接购买</option>
            <option value="assigned_only">仅指定用户购买</option>
          </select>
        </div>
        <div class="field"><label>是否上架展示</label><select id="np_show"><option value="1">是</option><option value="0">否</option></select></div>
        <div class="field"><label>是否允许新用户购买</label><select id="np_sell"><option value="1">是</option><option value="0">否</option></select></div>
        <div class="field"><label>是否允许已购用户续费</label><select id="np_renew"><option value="1">是</option><option value="0">否</option></select></div>
        <div class="field"><label>显示顺序（数字越小越靠前）</label><input id="np_sort" type="number" value="${plan?.sort ?? 0}"></div>
        <div class="field"><label>最低用户等级（0-4）</label><input id="np_min_trust" type="number" min="0" max="4" value="${plan?.min_trust_level ?? ''}" placeholder="留空=不限制"></div>
        <div class="field" style="grid-column: 1 / -1;" id="np_assigned_users_wrap">
          <label>可购买用户 ID（仅指定用户模式生效，多个用逗号）</label>
          <input id="np_access_user_ids" value="${escapeHtml(initialAccessUserIds)}" placeholder="例如 1,2,3">
        </div>
        <div class="field" style="grid-column: 1 / -1;" id="np_share_token_wrap">
          <label>专属购买口令（仅专属链接模式，留空自动生成）</label>
          <input id="np_share_token" value="${escapeHtml(initialShareToken)}" placeholder="例如 Abc12345xyz">
          <div class="row" style="margin-top:6px;">
            <input id="np_share_link_preview" readonly value="${escapeHtml(buildNodePlanShareLink(initialShareToken))}" placeholder="保存后生成专属链接">
            <button class="btn small" id="np_copy_share_link_btn" type="button">复制链接</button>
          </div>
        </div>
        <div class="field" style="grid-column: 1 / -1;"><label>节点（多选）</label>
          <div class="grid" style="grid-template-columns: repeat(2, minmax(0,1fr)); gap: 8px;">
            ${nodes.map(n => {
              const checked = asArray(plan?.node_ids).includes(n.id);
              return `<label class="notice" style="display:flex;gap:10px;align-items:center;"><input type="checkbox" class="np_node" value="${escapeHtml(String(n.id))}" ${checked ? 'checked' : ''}> <span>${escapeHtml(String(n.name || ''))} (#${escapeHtml(String(n.id))})</span></label>`;
            }).join('')}
          </div>
        </div>
        <div class="field">
          <label>允许试用（使用免费额度）</label>
          <select id="np_allow_trial">
            <option value="1">允许</option>
            <option value="0">不允许</option>
          </select>
        </div>
        <div class="field">
          <label>付费额度（GB，购买后可用）</label>
          <input id="np_paid_quota" type="number" min="0" value="${plan?.paid_quota_gb ?? plan?.transfer_enable ?? 0}">
          <label class="row" style="margin-top:6px;align-items:center;gap:8px;">
            <input id="np_unlimited_traffic" type="checkbox" ${initialUnlimitedTraffic ? 'checked' : ''}>
            <span class="muted">无限流量（购买后不限总流量）</span>
          </label>
        </div>
        <div class="field" style="grid-column: 1 / -1;">
          <label>试用免费额度（按用户等级，每月GB，作用于本套餐节点）</label>
          <div class="grid cols-2" id="np_free_quota_wrap">
            ${renderFreeQuotaInputFields('np_free_quota_', initialFreeQuota)}
          </div>
        </div>
        <div class="field" style="grid-column: 1 / -1;"><label>描述</label><textarea id="np_content" rows="5">${escapeHtml(String(plan?.content || ''))}</textarea></div>
        <div class="field" style="grid-column: 1 / -1;">
          <label>价格（元）</label>
          <div class="grid cols-2">
          ${PLAN_PRICE_FIELDS.map((item) => {
              const hasPrice = Object.prototype.hasOwnProperty.call(initialPrices, item.key);
              const rawFen = Number(initialPrices[item.key] || 0);
              const rawYuan = hasPrice ? (rawFen / 100).toFixed(2) : '';
              return `
                <div class="field">
                  <label>${item.label}</label>
                  <input id="np_price_${item.key}" type="number" min="0" step="0.01" value="${rawYuan}" placeholder="留空=不售卖">
                </div>
              `;
            }).join('')}
          </div>
        </div>
      </div>
      <div class="row end" style="margin-top: 12px;">
        <button class="btn primary" id="np_save">保存</button>
      </div>
    </div>
  `);

  qs('#np_show').value = String(plan?.show ? 1 : 0);
  qs('#np_sell').value = String(plan?.sell ? 1 : 0);
  qs('#np_renew').value = String(plan?.renew ? 1 : 0);
  qs('#np_allow_trial').value = initialAllowTrial ? '1' : '0';
  qs('#np_visibility_scope').value = initialVisibility;

  const syncPlanVisibilityUi = () => {
    const visibilityScope = normalizePlanVisibilityScope(qs('#np_visibility_scope')?.value);
    const showSelect = qs('#np_show');
    const assignedWrap = qs('#np_assigned_users_wrap');
    const shareWrap = qs('#np_share_token_wrap');
    if (assignedWrap) assignedWrap.classList.toggle('hidden', visibilityScope !== 'assigned_only');
    if (shareWrap) shareWrap.classList.toggle('hidden', visibilityScope !== 'link_only');
    if (showSelect) {
      if (visibilityScope === 'public') {
        showSelect.disabled = false;
      } else {
        showSelect.value = '0';
        showSelect.disabled = true;
      }
    }
  };
  const syncTrialUi = () => {
    const allowTrial = qs('#np_allow_trial')?.value === '1';
    const freeQuotaWrap = qs('#np_free_quota_wrap');
    if (freeQuotaWrap) freeQuotaWrap.classList.toggle('hidden', !allowTrial);
  };
  const syncUnlimitedTrafficUi = () => {
    const isUnlimited = Boolean(qs('#np_unlimited_traffic')?.checked);
    const paidQuotaInput = qs('#np_paid_quota');
    if (!paidQuotaInput) return;

    if (isUnlimited) {
      paidQuotaInput.dataset.prevValue = paidQuotaInput.value;
      paidQuotaInput.value = '';
      paidQuotaInput.disabled = true;
      paidQuotaInput.placeholder = '无限流量';
    } else {
      paidQuotaInput.disabled = false;
      paidQuotaInput.placeholder = '';
      if (paidQuotaInput.value === '' && paidQuotaInput.dataset.prevValue !== undefined) {
        paidQuotaInput.value = paidQuotaInput.dataset.prevValue;
      }
    }
  };
  const syncSharePreview = () => {
    const input = qs('#np_share_token');
    const preview = qs('#np_share_link_preview');
    if (!preview) return;
    preview.value = buildNodePlanShareLink(input?.value || '');
  };
  qs('#np_visibility_scope')?.addEventListener('change', syncPlanVisibilityUi);
  qs('#np_allow_trial')?.addEventListener('change', syncTrialUi);
  qs('#np_unlimited_traffic')?.addEventListener('change', syncUnlimitedTrafficUi);
  qs('#np_share_token')?.addEventListener('input', syncSharePreview);
  qs('#np_copy_share_link_btn')?.addEventListener('click', async () => {
    try {
      await copyText(qs('#np_share_link_preview')?.value || '');
      alert('专属购买链接已复制');
    } catch (e) {
      alert(e.message || '复制失败');
    }
  });
  syncPlanVisibilityUi();
  syncTrialUi();
  syncUnlimitedTrafficUi();
  syncSharePreview();

  qs('#np_save').addEventListener('click', async () => {
    try {
      const nodeIds = qsa('.np_node').filter(x => x.checked).map(x => Number(x.value));
      const prices = {};
      PLAN_PRICE_FIELDS.forEach((item) => {
        const raw = String(qs(`#np_price_${item.key}`)?.value || '').trim();
        if (!raw) return;
        const val = Number(raw);
        if (!Number.isFinite(val) || val < 0) {
          throw new Error(`${item.label}价格必须大于等于 0`);
        }
        prices[item.key] = Number(val.toFixed(2));
      });
      if (!Object.keys(prices).length) {
        throw new Error('至少设置一个购买价格');
      }
      const allowTrial = qs('#np_allow_trial').value === '1';
      const isUnlimitedTraffic = Boolean(qs('#np_unlimited_traffic')?.checked);
      const visibilityScope = normalizePlanVisibilityScope(qs('#np_visibility_scope').value);
      const freeQuota = allowTrial ? readFreeQuotaInputs('np_free_quota_') : null;
      const minTrustRaw = qs('#np_min_trust').value.trim();
      const accessUserIds = visibilityScope === 'assigned_only'
        ? parseIdListInput(qs('#np_access_user_ids').value)
        : [];
      const shareTokenRaw = visibilityScope === 'link_only'
        ? String(qs('#np_share_token').value || '').trim()
        : '';
      const payload = {
        name: qs('#np_name').value.trim(),
        content: qs('#np_content').value,
        prices,
        show: qs('#np_show').value === '1',
        sell: qs('#np_sell').value === '1',
        renew: qs('#np_renew').value === '1',
        sort: Number(qs('#np_sort').value),
        transfer_enable: isUnlimitedTraffic ? 0 : Number(qs('#np_paid_quota').value || 0),
        is_unlimited_traffic: isUnlimitedTraffic,
        node_ids: nodeIds,
        min_trust_level: minTrustRaw === '' ? null : Number(minTrustRaw),
        allow_trial: allowTrial,
        free_quota_gb_by_trust_level: freeQuota,
        visibility_scope: visibilityScope,
        access_user_ids: accessUserIds,
        share_token: shareTokenRaw || null,
      };
      if (visibilityScope === 'assigned_only' && !accessUserIds.length) {
        throw new Error('指定用户模式下，至少填写一个可购买用户 ID');
      }
      if (mode === 'new') {
        await apiFetch('/api/v1/user/node-plans', { method: 'POST', body: payload });
      } else {
        await apiFetch('/api/v1/user/node-plans/' + id, { method: 'PUT', body: payload });
      }
      location.hash = '#/node-plans';
    } catch (e) {
      renderNodePlanForm(mode, id, e.message || '保存失败');
    }
  });
}

async function renderRefunds(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const disputeEnabled = Boolean(store.me?.refund_dispute_enable);

  let items = [];
  try {
    const res = await apiFetch('/api/v1/user/refunds');
    items = asArray(res?.data);
  } catch (e) {
    items = [];
    errorText = e.message || errorText || '加载失败';
  }

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>我的退款</h2>
        <button class="btn primary" id="newRefundBtn">申请退款</button>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <table class="table" style="margin-top: 10px;">
        <thead><tr><th>ID</th><th>订单</th><th>套餐</th><th>状态</th><th>退回(分)</th><th>收费(分)</th><th></th></tr></thead>
        <tbody>
          ${items.map(r => html`
            <tr>
              <td>${r.id}</td>
              <td class="muted">${r.trade_no}</td>
              <td>${escapeHtml(r.plan?.name || r.plan_id)}</td>
              <td><span class="pill">${r.status}</span></td>
              <td class="muted">${r.refund_amount ?? '-'}</td>
              <td class="muted">${r.charged_amount ?? '-'}</td>
              <td class="row end"><a class="btn small" href="#/refund/${r.id}">查看</a></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
      <div class="muted" style="margin-top: 10px;">
        ${disputeEnabled
          ? '退款按节点套餐内流量使用比例计算；投票结束会自动裁定（超管可最终判定）。'
          : '退款按节点套餐内流量使用比例计算；当前已关闭争议投票。'}
      </div>
    </div>
  `);

  qs('#newRefundBtn').addEventListener('click', async () => {
    const tradeNo = prompt('订单 trade_no', '');
    if (!tradeNo) return;
    try {
      const reason = prompt('退款原因（可选）', '') || '';
      const evidence = prompt('证据（可选）', '') || '';
      const res = await apiFetch('/api/v1/user/refunds', { method: 'POST', body: { trade_no: tradeNo, reason, evidence } });
      const id = res?.data?.id;
      if (id) location.hash = '#/refund/' + id;
      else location.hash = '#/refunds';
    } catch (e) {
      renderRefunds(e.message || '申请失败');
    }
  });
}

async function renderRefundDetail(id, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const disputeEnabled = Boolean(store.me?.refund_dispute_enable);

  let r = null;
  try {
    const res = await apiFetch('/api/v1/user/refunds/' + encodeURIComponent(id));
    r = res?.data || null;
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '加载失败')}</div>`);
    return;
  }

  const ev = asArray(r?.evidences);
  const vc = r?.vote_counts || { approve: 0, deny: 0 };

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>退款 #${r.id}</h2>
        <a class="btn" href="#/refunds">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">订单</div><div class="v">${r.trade_no}</div>
        <div class="k">套餐</div><div class="v">${escapeHtml(r.plan?.name || r.plan_id)}</div>
        <div class="k">状态</div><div class="v"><span class="pill">${r.status}</span></div>
        <div class="k">退回(分)</div><div class="v">${r.refund_amount ?? '-'}</div>
        <div class="k">收费(分)</div><div class="v">${r.charged_amount ?? '-'}</div>
        <div class="k">用量(KB)</div><div class="v">${r.used_kb ?? '-'} / ${r.allowance_kb ?? '-'}</div>
        ${disputeEnabled
          ? `<div class="k">投票</div><div class="v">approve=${vc.approve} deny=${vc.deny} ${r.status === 'voting' ? `<a class="btn small" href="#/refund-vote/${r.id}">去投票</a>` : ''}</div>`
          : ''}
      </div>

      <div class="card" style="margin-top: 12px;">
        <h3>证据</h3>
        <div class="grid" style="gap: 8px; margin-top: 10px;">
          ${ev.map(e => html`
            <div class="notice">
              <div class="muted">${e.role} · user_id=${e.user_id} · ${e.created_at}</div>
              <div>${escapeHtml(String(e.content || '')).replaceAll('\\n','<br>')}</div>
            </div>
          `).join('')}
        </div>
        <div class="row end" style="margin-top: 10px;">
          <button class="btn" id="addEvidenceBtn">追加证据</button>
        </div>
      </div>
    </div>
  `);

  qs('#addEvidenceBtn').addEventListener('click', async () => {
    const content = prompt('证据内容', '');
    if (!content) return;
    try {
      await apiFetch(`/api/v1/user/refunds/${r.id}/evidence`, { method: 'POST', body: { content } });
      await renderRefundDetail(r.id);
    } catch (e) {
      await renderRefundDetail(r.id, e.message || '提交失败');
    }
  });
}

async function renderRefundVoteList(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  if (!Boolean(store.me?.refund_dispute_enable)) {
    location.hash = '#/refunds';
    return;
  }

  let items = [];
  try {
    const res = await apiFetch('/api/v1/user/refund-votes');
    items = asArray(res?.data);
  } catch (e) {
    items = [];
    errorText = e.message || errorText || '加载失败';
  }

  setView(html`
    <div class="card">
      <h2>争议投票</h2>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <table class="table" style="margin-top: 10px;">
        <thead><tr><th>案件</th><th>套餐</th><th>结束</th><th>状态</th><th></th></tr></thead>
        <tbody>
          ${items.map(r => html`
            <tr>
              <td>#${r.id}</td>
              <td>${escapeHtml(r.plan?.name || r.plan_id)}</td>
              <td class="muted">${r.voting_ends_at || '-'}</td>
              <td><span class="pill">${r.status}</span></td>
              <td class="row end"><a class="btn small" href="#/refund-vote/${r.id}">进入</a></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
      <div class="muted" style="margin-top: 10px;">所有成员可参与投票；投票结束后系统自动裁定（超管可覆盖）。</div>
    </div>
  `);
}

async function renderRefundVoteDetail(id, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  if (!Boolean(store.me?.refund_dispute_enable)) {
    location.hash = '#/refunds';
    return;
  }

  let r = null;
  try {
    const res = await apiFetch('/api/v1/user/refund-votes/' + encodeURIComponent(id));
    r = res?.data || null;
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '加载失败')}</div>`);
    return;
  }

  const ev = asArray(r?.evidences);
  const vc = r?.vote_counts || { approve: 0, deny: 0 };

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>投票：退款 #${r.id}</h2>
        <a class="btn" href="#/refund-votes">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">案件</div><div class="v">#${r.id}</div>
        <div class="k">套餐</div><div class="v">${escapeHtml(r.plan?.name || r.plan_id)}</div>
        <div class="k">结束</div><div class="v">${r.voting_ends_at || '-'}</div>
        <div class="k">票数</div><div class="v">approve=${vc.approve} deny=${vc.deny}</div>
        <div class="k">我的投票</div><div class="v">${r.my_vote || '-'}</div>
      </div>

      <div class="row" style="margin-top: 12px;">
        <button class="btn primary" id="voteApprove">投：同意退款</button>
        <button class="btn danger" id="voteDeny">投：拒绝退款</button>
        <button class="btn" id="voteEvidence">提交证据</button>
        <a class="btn" href="#/refund/${r.id}">详情</a>
      </div>

      <div class="card" style="margin-top: 12px;">
        <h3>证据</h3>
        <div class="grid" style="gap: 8px; margin-top: 10px;">
          ${ev.map(e => html`
            <div class="notice">
              <div class="muted">${e.role}${e.is_mine ? ' · 我' : ''} · ${e.created_at}</div>
              <div>${escapeHtml(String(e.content || '')).replaceAll('\\n','<br>')}</div>
            </div>
          `).join('')}
        </div>
      </div>
    </div>
  `);

  qs('#voteApprove').addEventListener('click', async () => {
    try {
      await apiFetch(`/api/v1/user/refund-votes/${r.id}`, { method: 'POST', body: { vote: 'approve' } });
      await renderRefundVoteDetail(r.id);
    } catch (e) {
      await renderRefundVoteDetail(r.id, e.message || '投票失败');
    }
  });
  qs('#voteDeny').addEventListener('click', async () => {
    try {
      await apiFetch(`/api/v1/user/refund-votes/${r.id}`, { method: 'POST', body: { vote: 'deny' } });
      await renderRefundVoteDetail(r.id);
    } catch (e) {
      await renderRefundVoteDetail(r.id, e.message || '投票失败');
    }
  });
  qs('#voteEvidence').addEventListener('click', async () => {
    const content = prompt('证据内容', '');
    if (!content) return;
    try {
      await apiFetch(`/api/v1/user/refund-votes/${r.id}/evidence`, { method: 'POST', body: { content } });
      await renderRefundVoteDetail(r.id);
    } catch (e) {
      await renderRefundVoteDetail(r.id, e.message || '提交失败');
    }
  });
}

async function renderNodeAdminRefunds(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  let items = [];
  try {
    const res = await apiFetch('/api/v1/user/node-admin/refunds');
    items = asArray(res?.data);
  } catch (e) {
    items = [];
    errorText = e.message || errorText || '加载失败';
  }

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>节点退款收件箱</h2>
        <a class="btn" href="#/node-admin">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <table class="table" style="margin-top: 10px;">
        <thead><tr><th>ID</th><th>订单</th><th>用户</th><th>状态</th><th>退回/收费(分)</th><th></th></tr></thead>
        <tbody>
          ${items.map(r => html`
            <tr>
              <td>${r.id}</td>
              <td class="muted">${r.trade_no}</td>
              <td class="muted">${r.user_id}</td>
              <td><span class="pill">${r.status}</span></td>
              <td class="muted">${r.refund_amount ?? '-'} / ${r.charged_amount ?? '-'}</td>
              <td class="row end"><a class="btn small" href="#/node-admin/refund/${r.id}">处理</a></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);
}

async function renderNodeAdminRefundDetail(id, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const disputeEnabled = Boolean(store.me?.refund_dispute_enable);

  let r = null;
  try {
    const res = await apiFetch('/api/v1/user/node-admin/refunds/' + encodeURIComponent(id));
    r = res?.data || null;
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '加载失败')}</div>`);
    return;
  }

  const ev = asArray(r?.evidences);
  const vc = r?.vote_counts || { approve: 0, deny: 0 };

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>处理退款 #${r.id}</h2>
        <a class="btn" href="#/node-admin/refunds">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">订单</div><div class="v">${r.trade_no}</div>
        <div class="k">用户</div><div class="v">${r.user_id}</div>
        <div class="k">状态</div><div class="v"><span class="pill">${r.status}</span></div>
        <div class="k">退回(分)</div><div class="v">${r.refund_amount ?? '-'}</div>
        <div class="k">收费(分)</div><div class="v">${r.charged_amount ?? '-'}</div>
        ${disputeEnabled ? `<div class="k">投票</div><div class="v">approve=${vc.approve} deny=${vc.deny}</div>` : ''}
      </div>

      <div class="row" style="margin-top: 12px;">
        <button class="btn primary" id="admRefundApprove">同意退款</button>
        <button class="btn danger" id="admRefundDeny">拒绝退款</button>
        ${disputeEnabled ? '<button class="btn" id="admRefundDispute">发起争议投票</button>' : ''}
        <button class="btn" id="admRefundEvidence">提交证据</button>
      </div>

      <div class="card" style="margin-top: 12px;">
        <h3>证据</h3>
        <div class="grid" style="gap: 8px; margin-top: 10px;">
          ${ev.map(e => html`
            <div class="notice">
              <div class="muted">${e.role} · user_id=${e.user_id} · ${e.created_at}</div>
              <div>${escapeHtml(String(e.content || '')).replaceAll('\\n','<br>')}</div>
            </div>
          `).join('')}
        </div>
      </div>
    </div>
  `);

  qs('#admRefundApprove').addEventListener('click', async () => {
    if (!confirm('确认同意退款？')) return;
    try {
      await apiFetch(`/api/v1/user/node-admin/refunds/${r.id}/approve`, { method: 'POST', body: {} });
      await renderNodeAdminRefundDetail(r.id);
    } catch (e) {
      await renderNodeAdminRefundDetail(r.id, e.message || '操作失败');
    }
  });
  qs('#admRefundDeny').addEventListener('click', async () => {
    const reason = prompt('拒绝原因（可选）', '') || '';
    try {
      await apiFetch(`/api/v1/user/node-admin/refunds/${r.id}/deny`, { method: 'POST', body: { reason } });
      await renderNodeAdminRefundDetail(r.id);
    } catch (e) {
      await renderNodeAdminRefundDetail(r.id, e.message || '操作失败');
    }
  });
  const disputeBtn = qs('#admRefundDispute');
  if (disputeBtn) {
    disputeBtn.addEventListener('click', async () => {
      const minutes = Number(prompt('投票时长（分钟）', '1440') || '1440');
      const evidence = prompt('发起争议时的证据（可选）', '') || '';
      try {
        await apiFetch(`/api/v1/user/node-admin/refunds/${r.id}/dispute`, { method: 'POST', body: { minutes, evidence } });
        await renderNodeAdminRefundDetail(r.id);
      } catch (e) {
        await renderNodeAdminRefundDetail(r.id, e.message || '操作失败');
      }
    });
  }
  qs('#admRefundEvidence').addEventListener('click', async () => {
    const content = prompt('证据内容', '');
    if (!content) return;
    try {
      await apiFetch(`/api/v1/user/node-admin/refunds/${r.id}/evidence`, { method: 'POST', body: { content } });
      await renderNodeAdminRefundDetail(r.id);
    } catch (e) {
      await renderNodeAdminRefundDetail(r.id, e.message || '提交失败');
    }
  });
}

async function renderNodeAdmin() {
  if (!requireAuth()) return;
  await loadMe();
  const disputeEnabled = Boolean(store.me?.refund_dispute_enable);

  const nodesRes = await apiFetch('/api/v1/user/server-nodes');
  const nodes = asArray(nodesRes?.data);

  const inboxRes = await apiFetch('/api/v1/user/node-admin/tickets');
  const inbox = asArray(inboxRes?.data);

  let refundInbox = [];
  try {
    const rr = await apiFetch('/api/v1/user/node-admin/refunds');
    refundInbox = asArray(rr?.data);
  } catch (e) {
    refundInbox = [];
  }

  setView(html`
    <div class="grid cols-2">
      <div class="card">
        <h2>节点统计</h2>
        <table class="table" style="margin-top: 10px;">
          <thead><tr><th>ID</th><th>名称</th><th>状态</th><th></th></tr></thead>
          <tbody>
            ${nodes.map(n => html`
              <tr>
                <td>${n.id}</td>
                <td>${escapeHtml(String(n.name || ''))}</td>
                <td>${pillStatus(n)}</td>
                <td class="row end">
                  <a class="btn small" href="#/node-admin/node/${n.id}">用户流量</a>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>
      <div class="card">
        <h2>节点工单收件箱</h2>
        <div class="muted">指派给你（你负责的节点）</div>
        <table class="table" style="margin-top: 10px;">
          <thead><tr><th>ID</th><th>节点</th><th>主题</th><th></th></tr></thead>
          <tbody>
            ${inbox.slice(0, 20).map(t => html`
              <tr>
                <td>${t.id}</td>
                <td class="muted">${t.node_id || '-'}</td>
                <td>${escapeHtml(String(t.subject || ''))}</td>
                <td class="row end"><a class="btn small" href="#/node-admin/ticket/${t.id}">处理</a></td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>节点退款收件箱</h2>
        <a class="btn" href="#/node-admin/refunds">全部</a>
      </div>
      <div class="muted">
        ${disputeEnabled ? '用户申请退款后，你可同意或发起争议投票。' : '用户申请退款后，你可同意或拒绝退款。'}
      </div>
      <table class="table" style="margin-top: 10px;">
        <thead><tr><th>ID</th><th>订单</th><th>用户</th><th>状态</th><th></th></tr></thead>
        <tbody>
          ${refundInbox.slice(0, 10).map(r => html`
            <tr>
              <td>${r.id}</td>
              <td class="muted">${r.trade_no}</td>
              <td class="muted">${r.user_id}</td>
              <td><span class="pill">${r.status}</span></td>
              <td class="row end"><a class="btn small" href="#/node-admin/refund/${r.id}">处理</a></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);
}

async function renderNodeAdminNodeUsers(nodeId, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const res = await apiFetch(`/api/v1/user/node-admin/server-nodes/${nodeId}/users-traffic?days=30`);
  const data = res?.data || {};
  const users = asArray(data.users);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>节点用户流量：${escapeHtml(String(data.node?.name || nodeId))}</h2>
        <a class="btn" href="#/node-admin">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <table class="table" style="margin-top: 10px;">
        <thead>
          <tr><th>User</th><th>总量(KB)</th><th>上行</th><th>下行</th><th>拉黑</th><th></th></tr>
        </thead>
        <tbody>
          ${users.map(u => html`
            <tr>
              <td class="muted">${u.user_id} · ${escapeHtml(u.email || '')}</td>
              <td>${u.total}</td>
              <td class="muted">${u.upload}</td>
              <td class="muted">${u.download}</td>
              <td>${u.is_blacklisted ? `<span class="pill bad">yes</span>` : `<span class="pill ok">no</span>`}</td>
              <td class="row end">
                ${u.is_blacklisted
                  ? `<button class="btn small" data-unblack="${u.user_id}">取消拉黑</button>`
                  : `<button class="btn small danger" data-black="${u.user_id}">拉黑</button>`
                }
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);

  qsa('button[data-black]').forEach(btn => btn.addEventListener('click', async () => {
    const uid = Number(btn.getAttribute('data-black'));
    const reason = prompt('拉黑原因（可选）', '');
    try {
      await apiFetch(`/api/v1/user/node-admin/server-nodes/${nodeId}/blacklist`, { method: 'POST', body: { user_id: uid, reason } });
      await renderNodeAdminNodeUsers(nodeId);
    } catch (e) {
      await renderNodeAdminNodeUsers(nodeId, e.message || '拉黑失败');
    }
  }));

  qsa('button[data-unblack]').forEach(btn => btn.addEventListener('click', async () => {
    const uid = Number(btn.getAttribute('data-unblack'));
    try {
      await apiFetch(`/api/v1/user/node-admin/server-nodes/${nodeId}/unblacklist`, { method: 'POST', body: { user_id: uid } });
      await renderNodeAdminNodeUsers(nodeId);
    } catch (e) {
      await renderNodeAdminNodeUsers(nodeId, e.message || '取消失败');
    }
  }));
}

async function renderNodeAdminTicket(id, errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  const res = await apiFetch('/api/v1/user/node-admin/ticket/detail?id=' + encodeURIComponent(id));
  const ticket = res?.data || null;

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>处理工单 #${ticket?.id}</h2>
        <a class="btn" href="#/node-admin">返回</a>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">节点</div><div class="v">${ticket?.node_id || '-'}</div>
        <div class="k">主题</div><div class="v">${escapeHtml(String(ticket?.subject || ''))}</div>
        <div class="k">状态</div><div class="v"><span class="pill">${ticket?.status}</span></div>
      </div>
      <div class="card" style="margin-top: 12px;">
        <h3>消息</h3>
        <div class="grid" style="gap: 8px; margin-top: 10px;">
          ${asArray(ticket?.message).map(m => html`
            <div class="notice ${m.is_me ? 'ok' : ''}">
              <div class="muted">user_id=${m.user_id} · ${m.created_at}</div>
              <div>${escapeHtml(String(m.message || '')).replaceAll('\n','<br>')}</div>
            </div>
          `).join('')}
        </div>
      </div>
      <div class="card" style="margin-top: 12px;">
        <h3>回复</h3>
        <textarea id="admReply" rows="4" placeholder="输入回复"></textarea>
        <div class="row end" style="margin-top: 10px;">
          <button class="btn primary" id="admReplyBtn">回复</button>
          <button class="btn danger" id="admCloseBtn">关闭</button>
        </div>
      </div>
    </div>
  `);

  qs('#admReplyBtn').addEventListener('click', async () => {
    try {
      await apiFetch('/api/v1/user/node-admin/ticket/reply', { method: 'POST', body: { id: Number(id), message: qs('#admReply').value } });
      await renderNodeAdminTicket(id);
    } catch (e) {
      await renderNodeAdminTicket(id, e.message || '回复失败');
    }
  });

  qs('#admCloseBtn').addEventListener('click', async () => {
    if (!confirm('确认关闭工单？')) return;
    await apiFetch('/api/v1/user/node-admin/ticket/close', { method: 'POST', body: { id: Number(id) } });
    location.hash = '#/node-admin';
  });
}

async function renderAudit() {
  if (!requireAuth()) return;
  await loadMe();

  const logs = await apiFetch('/api/v1/user/audit-logs?limit=100');
  const items = asArray(logs?.data);

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>我的审计日志</h2>
        <div class="muted">最近 100 条</div>
      </div>
      <table class="table" style="margin-top: 10px;">
        <thead>
          <tr><th>ID</th><th>节点</th><th>动作</th><th>目标</th><th>IP</th><th>时间</th></tr>
        </thead>
        <tbody>
          ${items.map(l => html`
            <tr>
              <td>${l.id}</td>
              <td class="muted">${escapeHtml(l.node?.name || l.node_id)}</td>
              <td><span class="pill">${escapeHtml(l.action_taken)}</span></td>
              <td class="muted">${escapeHtml(l.target_domain || l.target_protocol || '-')}</td>
              <td class="muted">${escapeHtml(l.ip_address)}</td>
              <td class="muted">${escapeHtml(l.created_at)}</td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `);
}

async function renderAdminTemplateEditor(templateType = 'singbox', messageText = '', messageKind = '') {
  if (!requireAuth()) return;
  const me = await loadMe();
  if (!me?.is_super_admin) {
    setView(`<div class="notice error">需要超级管理员权限</div>`);
    return;
  }

  const securePath = String(me?.secure_path || '').trim();
  if (!securePath) {
    setView(`<div class="notice error">未找到后台路径，无法编辑订阅模板</div>`);
    return;
  }

  const editor = getSubscribeTemplateEditor(templateType);
  const buildV2 = (endpoint) => `/api/v2/${encodeURIComponent(securePath)}/${String(endpoint || '').replace(/^\/+/, '')}`;

  let config = {};
  try {
    const cfg = await apiFetch(buildV2('config/fetch'));
    config = cfg?.data || {};
  } catch (e) {
    setView(`<div class="notice error">${escapeHtml(e.message || '读取配置失败')}</div>`);
    return;
  }

  const subscribeTemplateCfg = config?.subscribe_template || {};
  const templateValue = String(subscribeTemplateCfg?.[editor.configKey] || '');

  setView(html`
    <div class="card template-editor-shell">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>代理订阅模板编辑</h2>
        <a class="btn" href="#/admin/proxy-templates">返回超管页面</a>
      </div>
      ${messageText
        ? `<div class="notice ${messageKind === 'error' ? 'error' : (messageKind === 'ok' ? 'ok' : '')}" style="margin-top:10px;">${escapeHtml(messageText)}</div>`
        : ''}
      <div class="row" style="margin-top:12px;">
        ${SUBSCRIBE_TEMPLATE_EDITORS.map((item) => `
          <a class="btn ${item.key === editor.key ? 'primary' : ''}" href="#/admin/template/${item.key}">
            ${escapeHtml(item.label)}
          </a>
        `).join('')}
      </div>
      <div class="muted" style="margin-top:10px;">
        当前编辑：<strong>${escapeHtml(editor.label)}</strong>（${editor.format.toUpperCase()}）。
        该页面使用完整宽度，便于长模板查看与修改。
      </div>
      <div class="field" style="margin-top:12px;">
        <label>${escapeHtml(editor.label)} 模板内容</label>
        <textarea id="adm_tpl_editor_textarea" rows="30" style="min-height:68vh;">${escapeHtml(templateValue)}</textarea>
      </div>
      <div class="row end" style="margin-top:12px;">
        ${editor.format === 'json' ? '<button class="btn" id="admTplFormatBtn">格式化 JSON</button>' : ''}
        <button class="btn" id="admTplResetBtn">恢复为当前线上值</button>
        <button class="btn primary" id="admTplSaveBtn">保存模板</button>
      </div>
    </div>
  `);

  const area = qs('#adm_tpl_editor_textarea');
  const fmtBtn = qs('#admTplFormatBtn');
  if (fmtBtn && area) {
    fmtBtn.addEventListener('click', () => {
      try {
        const parsed = JSON.parse(String(area.value || '{}'));
        area.value = JSON.stringify(parsed, null, 2);
        alert('JSON 已格式化');
      } catch (_) {
        alert('JSON 格式不正确，请先修正后再保存');
      }
    });
  }

  const resetBtn = qs('#admTplResetBtn');
  if (resetBtn && area) {
    resetBtn.addEventListener('click', () => {
      area.value = templateValue;
    });
  }

  const saveBtn = qs('#admTplSaveBtn');
  if (saveBtn && area) {
    saveBtn.addEventListener('click', async () => {
      try {
        const nextValue = String(area.value || '');
        if (editor.format === 'json') {
          JSON.parse(nextValue || '{}');
        }
        await apiFetch(buildV2('config/save'), {
          method: 'POST',
          body: { [editor.configKey]: nextValue },
        });
        await renderAdminTemplateEditor(editor.key, `${editor.label} 模板已保存`, 'ok');
      } catch (e) {
        await renderAdminTemplateEditor(editor.key, e.message || '保存失败', 'error');
      }
    });
  }
}

function buildCommandCenterEmpty(message) {
  return `<div class="command-empty">${escapeHtml(message || '暂无数据')}</div>`;
}

function buildCommandCenterStatusBadge(label, tone = 'neutral') {
  return `<span class="command-badge ${escapeHtml(tone)}">${escapeHtml(label)}</span>`;
}

function buildCommandCenterAreaChart(points) {
  const rows = asArray(points);
  if (!rows.length) return buildCommandCenterEmpty('最近 7 天还没有节点流量记录');

  const width = 960;
  const height = 286;
  const left = 32;
  const right = 22;
  const top = 26;
  const bottom = 34;
  const chartWidth = width - left - right;
  const chartHeight = height - top - bottom;
  const maxValue = Math.max(1, ...rows.map((item) => Number(item?.total_kb || 0)));
  const step = rows.length > 1 ? chartWidth / (rows.length - 1) : chartWidth;
  const xFor = (idx) => left + step * idx;
  const yFor = (value) => top + (chartHeight - ((Number(value || 0) / maxValue) * chartHeight));
  const linePoints = rows.map((item, idx) => `${xFor(idx).toFixed(2)},${yFor(item?.total_kb || 0).toFixed(2)}`).join(' ');
  const areaPoints = `${left},${height - bottom} ${linePoints} ${left + chartWidth},${height - bottom}`;
  const gradientId = 'commandCenterTrafficGradient';
  const strokeId = 'commandCenterTrafficStroke';
  const ticks = [0, 0.33, 0.66, 1].map((ratio) => {
    const value = Math.round(maxValue * ratio);
    return { value, y: yFor(value) };
  });

  return html`
    <div class="command-chart-shell">
      <svg viewBox="0 0 ${width} ${height}" class="command-traffic-svg" role="img" aria-label="最近 7 天流量趋势">
        <defs>
          <linearGradient id="${gradientId}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="rgba(56,189,248,0.42)"></stop>
            <stop offset="100%" stop-color="rgba(14,165,233,0.02)"></stop>
          </linearGradient>
          <linearGradient id="${strokeId}" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0%" stop-color="#67e8f9"></stop>
            <stop offset="100%" stop-color="#60a5fa"></stop>
          </linearGradient>
        </defs>
        <rect x="0" y="0" width="${width}" height="${height}" rx="22" fill="rgba(3, 12, 26, 0.86)"></rect>
        ${ticks.map((tick) => `
          <g>
            <line x1="${left}" y1="${tick.y}" x2="${width - right}" y2="${tick.y}" stroke="rgba(148, 163, 184, 0.14)" stroke-dasharray="6 8"></line>
            <text x="4" y="${tick.y + 4}" fill="rgba(148, 163, 184, 0.78)" font-size="11">${escapeHtml(formatTrafficKb(tick.value))}</text>
          </g>
        `).join('')}
        <line x1="${left}" y1="${top}" x2="${left}" y2="${height - bottom}" stroke="rgba(148, 163, 184, 0.2)"></line>
        <line x1="${left}" y1="${height - bottom}" x2="${width - right}" y2="${height - bottom}" stroke="rgba(148, 163, 184, 0.2)"></line>
        <polygon points="${areaPoints}" fill="url(#${gradientId})"></polygon>
        <polyline points="${linePoints}" fill="none" stroke="url(#${strokeId})" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"></polyline>
        ${rows.map((item, idx) => `
          <g>
            <circle cx="${xFor(idx).toFixed(2)}" cy="${yFor(item?.total_kb || 0).toFixed(2)}" r="4.5" fill="#67e8f9" stroke="rgba(2, 6, 23, 0.92)" stroke-width="2"></circle>
            <text x="${xFor(idx).toFixed(2)}" y="${height - 10}" text-anchor="middle" fill="rgba(203, 213, 225, 0.76)" font-size="11">${escapeHtml(String(item?.date || '').slice(5))}</text>
          </g>
        `).join('')}
      </svg>
    </div>
  `;
}

function buildCommandCenterSparkline(samples, width = 260, height = 74) {
  const rows = asArray(samples);
  if (!rows.length) return '<div class="command-spark-empty">暂无 TCPing 采样</div>';

  const left = 8;
  const right = 8;
  const top = 10;
  const bottom = 14;
  const chartWidth = width - left - right;
  const chartHeight = height - top - bottom;
  const reachable = rows.filter((item) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)));
  const maxLatency = Math.max(40, ...reachable.map((item) => Number(item?.latency_ms || 0)));
  const step = rows.length > 1 ? chartWidth / (rows.length - 1) : chartWidth;
  const xFor = (idx) => left + step * idx;
  const yFor = (value) => top + (chartHeight - ((Number(value || 0) / maxLatency) * chartHeight));
  const linePoints = rows
    .map((item, idx) => (item?.is_reachable && Number.isFinite(Number(item?.latency_ms))
      ? `${xFor(idx).toFixed(2)},${yFor(item.latency_ms).toFixed(2)}`
      : null))
    .filter(Boolean)
    .join(' ');
  const offlineMarkers = rows
    .map((item, idx) => (!item?.is_reachable ? {
      x: xFor(idx),
      y: height - 10,
    } : null))
    .filter(Boolean);

  return html`
    <svg viewBox="0 0 ${width} ${height}" class="command-node-sparkline" role="img" aria-label="TCPing latency sparkline">
      <line x1="${left}" y1="${height - bottom}" x2="${width - right}" y2="${height - bottom}" stroke="rgba(148, 163, 184, 0.16)"></line>
      <line x1="${left}" y1="${top}" x2="${left}" y2="${height - bottom}" stroke="rgba(148, 163, 184, 0.1)"></line>
      ${linePoints ? `<polyline points="${linePoints}" fill="none" stroke="rgba(34,211,238,0.94)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"></polyline>` : ''}
      ${reachable.map((item, idx) => `
        <circle cx="${xFor(rows.indexOf(item)).toFixed(2)}" cy="${yFor(item.latency_ms).toFixed(2)}" r="2.5" fill="rgba(103,232,249,0.88)"></circle>
      `).join('')}
      ${offlineMarkers.map((dot) => `<circle cx="${dot.x.toFixed(2)}" cy="${dot.y.toFixed(2)}" r="3" fill="rgba(248,113,113,0.92)"></circle>`).join('')}
    </svg>
  `;
}

function buildCommandCenterBars(items, {
  valueKey = 'value',
  labelKey = 'label',
  emptyText = '暂无数据',
  accent = 'cyan',
  subtitle = null,
  valueFormatter = (value) => formatCompactNumber(value),
} = {}) {
  const rows = asArray(items);
  if (!rows.length) return buildCommandCenterEmpty(emptyText);

  const maxValue = Math.max(1, ...rows.map((item) => Number(item?.[valueKey] || 0)));
  return html`
    <div class="command-bars">
      ${rows.map((item) => {
        const value = Number(item?.[valueKey] || 0);
        const width = Math.max(4, (value / maxValue) * 100);
        const label = item?.[labelKey] ?? item?.label ?? item?.name ?? '-';
        const sub = typeof subtitle === 'function' ? subtitle(item) : '';
        return `
          <div class="command-bar-row">
            <div class="command-bar-copy">
              <strong>${escapeHtml(label)}</strong>
              ${sub ? `<span>${escapeHtml(sub)}</span>` : ''}
            </div>
            <div class="command-bar-track">
              <div class="command-bar-fill ${escapeHtml(accent)}" style="width:${width.toFixed(2)}%"></div>
            </div>
            <div class="command-bar-value">${escapeHtml(valueFormatter(value, item))}</div>
          </div>
        `;
      }).join('')}
    </div>
  `;
}

function buildCommandCenterDashboard(payload = {}, me = {}) {
  const overview = payload?.overview || {};
  const system = payload?.system || {};
  const trend = asArray(payload?.traffic_trend);
  const topUsers = asArray(payload?.top_users);
  const watchlist = asArray(payload?.node_watchlist);
  const protocolDistribution = asArray(payload?.protocol_distribution);
  const regionDistribution = asArray(payload?.region_distribution);
  const tickets = asArray(payload?.tickets);
  const refunds = asArray(payload?.refunds);
  const tcpingAgents = asArray(payload?.tcping_agents);
  const tcpingAlerts = asArray(payload?.tcping_alerts);
  const auditStream = asArray(payload?.audit_stream);

  const totalNodes = Number(overview?.total_nodes || 0);
  const onlineNodes = Number(overview?.online_nodes || 0);
  const nodeOnlineRatio = totalNodes > 0 ? (onlineNodes / totalNodes) * 100 : 0;
  const alertCount = Number(overview?.tcping_alerts_active || 0);
  const schedulerTone = system?.schedule_ok ? 'ok' : 'bad';
  const horizonTone = system?.horizon?.available ? (system?.horizon?.ok ? 'ok' : 'bad') : 'neutral';
  const alertTone = alertCount > 0 ? 'warn' : 'ok';
  const logTone = Number(system?.logs?.errors_last_24h || 0) > 0 ? 'warn' : 'ok';

  const metrics = [
    {
      label: '用户总量',
      value: formatCompactNumber(overview?.total_users || 0),
      meta: `实时活跃 ${formatCompactNumber(overview?.live_users || 0)}`,
      tone: 'cyan',
    },
    {
      label: '活跃订阅',
      value: formatCompactNumber(overview?.active_subscriptions || 0),
      meta: `今日使用 ${formatCompactNumber(overview?.traffic_today_unique_users || 0)} 用户`,
      tone: 'blue',
    },
    {
      label: '在线节点',
      value: `${onlineNodes}/${totalNodes || 0}`,
      meta: `在线率 ${formatPercent(nodeOnlineRatio)}`,
      tone: 'emerald',
    },
    {
      label: '高压节点',
      value: formatCompactNumber(overview?.traffic_hot_nodes || 0),
      meta: `维护中 ${formatCompactNumber(overview?.maintenance_nodes || 0)}`,
      tone: 'amber',
    },
    {
      label: 'TCPing 告警',
      value: formatCompactNumber(alertCount),
      meta: `监控覆盖 ${formatCompactNumber(overview?.tcping_enabled_nodes || 0)} 节点`,
      tone: 'rose',
    },
    {
      label: '探针在线',
      value: `${formatCompactNumber(overview?.tcping_agents_online || 0)}/${formatCompactNumber(overview?.tcping_agents_total || 0)}`,
      meta: '独立于 V2bX 的探活体系',
      tone: 'violet',
    },
    {
      label: '待处理事务',
      value: `${formatCompactNumber(overview?.open_tickets || 0)} / ${formatCompactNumber(overview?.pending_refunds || 0)}`,
      meta: '工单 / 退款',
      tone: 'orange',
    },
    {
      label: '今日流量',
      value: formatTrafficKb(overview?.traffic_today_kb || 0),
      meta: `24h 收入 ${formatMoneyCent(overview?.revenue_24h_amount || 0)}`,
      tone: 'teal',
    },
  ];

  return html`
    <div class="command-surface">
      <section class="command-hero-panel">
        <div class="command-hero-copy">
          <div class="command-kicker">SUPER ADMIN INTERNAL COMMAND GRID</div>
          <h2>NotXboard Hypervision Grid</h2>
          <p>节点在线状态、TCPing 告警、真实用户流量、退款与审计流汇聚到同一块指挥屏。因为仅限内部使用，邮件、Linux DO 名称与套餐信息全部原样展示。</p>
          <div class="command-hero-badges">
            ${buildCommandCenterStatusBadge(system?.schedule_ok ? 'Scheduler 正常' : 'Scheduler 异常', schedulerTone)}
            ${buildCommandCenterStatusBadge(system?.horizon?.available ? (system?.horizon?.ok ? 'Horizon 正常' : 'Horizon 暂停') : 'Horizon 未启用', horizonTone)}
            ${buildCommandCenterStatusBadge(alertCount > 0 ? `活跃告警 ${alertCount}` : '无活跃告警', alertTone)}
            ${buildCommandCenterStatusBadge(Number(system?.logs?.errors_last_24h || 0) > 0 ? `24h 错误 ${system.logs.errors_last_24h}` : '24h 无错误日志', logTone)}
          </div>
        </div>
        <div class="command-focus-grid">
          <div class="command-focus-card">
            <span>NODE UPTIME</span>
            <strong>${formatPercent(nodeOnlineRatio)}</strong>
            <small>${onlineNodes} / ${totalNodes || 0} 节点在线</small>
          </div>
          <div class="command-focus-card">
            <span>LIVE THROUGHPUT</span>
            <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_download_bps || 0))}</strong>
            <small>下行主导，上传 ${escapeHtml(formatRatePerSecond(overview?.throughput_upload_bps || 0))}</small>
          </div>
          <div class="command-focus-card">
            <span>USER FOOTPRINT</span>
            <strong>${escapeHtml(formatCompactNumber(overview?.traffic_today_unique_users || 0))}</strong>
            <small>今日实际使用节点的订阅用户</small>
          </div>
        </div>
      </section>

      <section class="command-metrics-grid">
        ${metrics.map((metric) => `
          <article class="command-metric-card ${escapeHtml(metric.tone)}">
            <span>${escapeHtml(metric.label)}</span>
            <strong>${escapeHtml(metric.value)}</strong>
            <small>${escapeHtml(metric.meta)}</small>
          </article>
        `).join('')}
      </section>

      <section class="command-layout">
        <article class="command-panel command-span-8">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">NETWORK FLOW</div>
              <h3>最近 7 天流量热区</h3>
            </div>
            <div class="command-inline-stats">
              <div class="command-inline-stat">
                <span>今日流量</span>
                <strong>${escapeHtml(formatTrafficKb(overview?.traffic_today_kb || 0))}</strong>
              </div>
              <div class="command-inline-stat">
                <span>24h 完成订单</span>
                <strong>${escapeHtml(formatCompactNumber(overview?.completed_orders_24h || 0))}</strong>
              </div>
              <div class="command-inline-stat">
                <span>实时下行</span>
                <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_download_bps || 0))}</strong>
              </div>
            </div>
          </div>
          ${buildCommandCenterAreaChart(trend)}
        </article>

        <article class="command-panel command-span-4">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">SYSTEM HEALTH</div>
              <h3>运行栈体征</h3>
            </div>
          </div>
          <div class="command-health-list">
            <div class="command-health-item">
              <div>
                <strong>计划任务</strong>
                <span>最近运行 ${escapeHtml(formatAnyTimestamp(system?.schedule_last_runtime))}</span>
              </div>
              ${buildCommandCenterStatusBadge(system?.schedule_ok ? '正常' : '异常', schedulerTone)}
            </div>
            <div class="command-health-item">
              <div>
                <strong>Horizon</strong>
                <span>${system?.horizon?.available ? `Master ${formatCompactNumber(system?.horizon?.master_count || 0)} / 暂停 ${formatCompactNumber(system?.horizon?.paused_masters || 0)}` : '当前环境未启用'}</span>
              </div>
              ${buildCommandCenterStatusBadge(system?.horizon?.available ? (system?.horizon?.ok ? '运行中' : '告警') : 'N/A', horizonTone)}
            </div>
            <div class="command-health-item">
              <div>
                <strong>日志压力</strong>
                <span>Info ${formatCompactNumber(system?.logs?.info || 0)} / Warn ${formatCompactNumber(system?.logs?.warning || 0)} / Error ${formatCompactNumber(system?.logs?.error || 0)}</span>
              </div>
              ${buildCommandCenterStatusBadge(Number(system?.logs?.errors_last_24h || 0) > 0 ? '需要关注' : '平稳', logTone)}
            </div>
          </div>
          <div class="command-log-grid">
            <div><span>24h 错误</span><strong>${escapeHtml(formatCompactNumber(system?.logs?.errors_last_24h || 0))}</strong></div>
            <div><span>24h 警告</span><strong>${escapeHtml(formatCompactNumber(system?.logs?.warnings_last_24h || 0))}</strong></div>
            <div><span>总日志</span><strong>${escapeHtml(formatCompactNumber(system?.logs?.total || 0))}</strong></div>
            <div><span>当前账号</span><strong>${escapeHtml(me?.email || '-')}</strong></div>
          </div>
        </article>

        <article class="command-panel command-span-6">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">USER PRESSURE</div>
              <h3>高流量用户榜</h3>
            </div>
          </div>
          ${topUsers.length ? `
            <div class="command-user-list">
              ${topUsers.map((user, index) => {
                const max = Math.max(1, ...topUsers.map((item) => Number(item?.traffic_kb || 0)));
                const width = Math.max(8, ((Number(user?.traffic_kb || 0) / max) * 100));
                return `
                  <div class="command-user-row">
                    <div class="command-rank">#${String(index + 1).padStart(2, '0')}</div>
                    <div class="command-user-identity">
                      <strong>${escapeHtml(user?.display_name || user?.email || '-')}</strong>
                      <span>${escapeHtml(user?.email || '-')}</span>
                    </div>
                    <div class="command-user-bar">
                      <div class="command-user-bar-fill" style="width:${width.toFixed(2)}%"></div>
                    </div>
                    <div class="command-user-metrics">
                      <strong>${escapeHtml(formatTrafficKb(user?.traffic_kb || 0))}</strong>
                      <span>${escapeHtml(user?.plan_name || '-')}${user?.subscription_expired_at ? ` · 到期 ${escapeHtml(formatAnyTimestamp(user.subscription_expired_at))}` : ''}</span>
                    </div>
                  </div>
                `;
              }).join('')}
            </div>
          ` : buildCommandCenterEmpty('当前窗口暂无用户流量排行')}
        </article>

        <article class="command-panel command-span-6">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">TOPOLOGY MIX</div>
              <h3>协议与地区分布</h3>
            </div>
          </div>
          <div class="command-split-grid">
            <div>
              <h4>协议占比</h4>
              ${buildCommandCenterBars(protocolDistribution.map((item) => ({
                label: item?.label || item?.protocol || '-',
                value: item?.total || 0,
                online: item?.online || 0,
              })), {
                valueKey: 'value',
                labelKey: 'label',
                accent: 'cyan',
                emptyText: '暂无节点协议数据',
                subtitle: (item) => `在线 ${formatCompactNumber(item?.online || 0)}`,
                valueFormatter: (value) => `${formatCompactNumber(value)} 个`,
              })}
            </div>
            <div>
              <h4>地区热度</h4>
              ${buildCommandCenterBars(regionDistribution.map((item) => ({
                label: item?.location_name || item?.location_code || '-',
                value: item?.total || 0,
                online: item?.online || 0,
              })), {
                valueKey: 'value',
                labelKey: 'label',
                accent: 'violet',
                emptyText: '暂无节点地区数据',
                subtitle: (item) => `在线 ${formatCompactNumber(item?.online || 0)}`,
                valueFormatter: (value) => `${formatCompactNumber(value)} 个`,
              })}
            </div>
          </div>
        </article>

        <article class="command-panel command-span-12">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">NODE WATCHLIST</div>
              <h3>节点关注列表</h3>
            </div>
          </div>
          ${watchlist.length ? `
            <div class="command-node-grid">
              ${watchlist.map((node) => {
                const nodeMonitorHref = `#/nodes/${encodeURIComponent(String(node?.id || ''))}/monitor?hours=24`;
                const progress = Math.max(0, Math.min(100, Number(node?.traffic_usage_percentage || 0)));
                const meterClass = progress >= 90 ? 'danger' : (progress >= 70 ? 'warn' : 'ok');
                return `
                  <article class="command-node-card">
                    <div class="command-node-head">
                      <div>
                        <div class="command-node-title">
                          <h4>${escapeHtml(node?.name || '-')}</h4>
                          ${buildCommandCenterStatusBadge(node?.protocol_label || node?.protocol || '-', 'neutral')}
                        </div>
                        <div class="command-node-subline">${escapeHtml(node?.host || '-')} : ${escapeHtml(node?.port || '-')} · ${escapeHtml(node?.location_name || '-')}</div>
                      </div>
                      <div class="command-node-badges">
                        ${buildCommandCenterStatusBadge(node?.online_status === 'online' ? '上报在线' : '上报离线', node?.online_status === 'online' ? 'ok' : 'bad')}
                        ${buildCommandCenterStatusBadge(node?.tcping_status === 'unsupported'
                          ? '不支持 UDP 探测'
                          : (node?.tcping_enabled ? (node?.tcping_status === 'offline' ? 'TCPing 异常' : (node?.tcping_status === 'online' ? 'TCPing 正常' : 'TCPing 等待')) : '未启用 TCPing'),
                        node?.tcping_status === 'unsupported'
                          ? 'neutral'
                          : (node?.tcping_enabled ? (node?.tcping_status === 'offline' ? 'warn' : 'ok') : 'neutral'))}
                        ${node?.active_alert ? buildCommandCenterStatusBadge(`告警 ${node.active_alert.count || 1}`, 'rose') : ''}
                      </div>
                    </div>
                    <div class="command-node-owner">${escapeHtml(node?.owner_name || '-')} · ${escapeHtml(node?.owner_email || '-')}</div>
                    <div class="command-node-spark">${buildCommandCenterSparkline(node?.tcping_samples)}</div>
                    <div class="command-node-meter">
                      <div class="command-node-meter-top">
                        <span>节点总流量</span>
                        <strong>${node?.traffic_limit_kb > 0 ? `${escapeHtml(formatTrafficKb(node?.traffic_used_kb || 0))} / ${escapeHtml(formatTrafficKb(node?.traffic_limit_kb || 0))}` : `${escapeHtml(formatTrafficKb(node?.traffic_used_kb || 0))} / 无限`}</strong>
                      </div>
                      <div class="command-meter ${meterClass}">
                        <div class="command-meter-fill" style="width:${node?.traffic_limit_kb > 0 ? progress.toFixed(2) : 100}%"></div>
                      </div>
                      <div class="command-node-meter-bottom">${node?.traffic_limit_kb > 0 ? `压力 ${escapeHtml(formatPercent(progress))}` : '未设置节点总流量上限'}</div>
                    </div>
                    <div class="command-node-stats">
                      <div><span>在线用户</span><strong>${escapeHtml(formatCompactNumber(node?.online_users || 0))}</strong></div>
                      <div><span>活跃连接</span><strong>${escapeHtml(formatCompactNumber(node?.active_connections || 0))}</strong></div>
                      <div><span>7 天流量</span><strong>${escapeHtml(formatTrafficKb(node?.weekly_traffic_kb || 0))}</strong></div>
                      <div><span>最近延迟</span><strong>${escapeHtml(formatLatencyMs(node?.tcping_last_latency_ms))}</strong></div>
                    </div>
                    <div class="command-node-footer">
                      <div class="command-node-alert">${node?.active_alert?.latest_error ? escapeHtml(node.active_alert.latest_error) : (node?.tcping_last_error ? escapeHtml(node.tcping_last_error) : '未检测到最新异常说明')}</div>
                      <a class="btn small" href="${nodeMonitorHref}">查看曲线</a>
                    </div>
                  </article>
                `;
              }).join('')}
            </div>
          ` : buildCommandCenterEmpty('当前没有节点可展示')}
        </article>

        <article class="command-panel command-span-4">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">TCPING ALERTS</div>
              <h3>活跃告警</h3>
            </div>
          </div>
          ${tcpingAlerts.length ? tcpingAlerts.map((alert) => `
            <div class="command-feed-item">
              <div class="command-feed-top">
                <strong>${escapeHtml(alert?.node_name || '-')}</strong>
                ${buildCommandCenterStatusBadge(alert?.status || 'active', 'rose')}
              </div>
              <div class="command-feed-meta">${escapeHtml(alert?.user_email || '-')} · ${escapeHtml(alert?.node_location_name || '-')} · ${escapeHtml(alert?.node_protocol || '-')}</div>
              <div class="command-feed-sub">${escapeHtml(formatDurationShort(alert?.duration_seconds || 0))} · ${escapeHtml(alert?.latest_error || '无错误信息')}</div>
            </div>
          `).join('') : buildCommandCenterEmpty('暂无活跃 TCPing 告警')}
        </article>

        <article class="command-panel command-span-4">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">PROBES</div>
              <h3>探针健康</h3>
            </div>
          </div>
          ${tcpingAgents.length ? tcpingAgents.map((agent) => `
            <div class="command-feed-item">
              <div class="command-feed-top">
                <strong>${escapeHtml(agent?.name || '-')}</strong>
                ${buildCommandCenterStatusBadge(agent?.is_online ? '在线' : '离线', agent?.is_online ? 'ok' : 'warn')}
              </div>
              <div class="command-feed-meta">${escapeHtml(agent?.owner_name || '-')} · ${escapeHtml(agent?.owner_email || '-')}</div>
              <div class="command-feed-sub">心跳 ${escapeHtml(formatAnyTimestamp(agent?.last_heartbeat_at))} · 同步 ${escapeHtml(formatAnyTimestamp(agent?.last_sync_at))}</div>
            </div>
          `).join('') : buildCommandCenterEmpty('暂无探针数据')}
        </article>

        <article class="command-panel command-span-4">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">SUPPORT QUEUE</div>
              <h3>工单池</h3>
            </div>
          </div>
          ${tickets.length ? tickets.map((ticket) => `
            <div class="command-feed-item">
              <div class="command-feed-top">
                <strong>${escapeHtml(ticket?.subject || '-')}</strong>
                ${buildCommandCenterStatusBadge(Number(ticket?.status || 0) === 0 ? 'OPEN' : 'CLOSED', Number(ticket?.status || 0) === 0 ? 'warn' : 'neutral')}
              </div>
              <div class="command-feed-meta">${escapeHtml(ticket?.user_email || '-')} · ${escapeHtml(ticket?.node_name || '-')}</div>
              <div class="command-feed-sub">更新 ${escapeHtml(formatAnyTimestamp(ticket?.updated_at))} · 负责人 ${escapeHtml(ticket?.assigned_admin_email || '-')}</div>
            </div>
          `).join('') : buildCommandCenterEmpty('当前没有开启工单')}
        </article>

        <article class="command-panel command-span-6">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">REFUND FLOW</div>
              <h3>退款流程</h3>
            </div>
          </div>
          ${refunds.length ? refunds.map((refund) => `
            <div class="command-feed-item">
              <div class="command-feed-top">
                <strong>${escapeHtml(refund?.user_email || '-')}</strong>
                ${buildCommandCenterStatusBadge(refund?.status || 'pending', refund?.status === 'voting' ? 'warn' : 'orange')}
              </div>
              <div class="command-feed-meta">${escapeHtml(refund?.plan_name || '-')} · trade_no ${escapeHtml(refund?.trade_no || '-')}</div>
              <div class="command-feed-sub">申请 ${escapeHtml(formatMoneyCent(refund?.refund_amount || 0))} / 原金额 ${escapeHtml(formatMoneyCent(refund?.gateway_amount || 0))} · 更新 ${escapeHtml(formatAnyTimestamp(refund?.updated_at))}</div>
            </div>
          `).join('') : buildCommandCenterEmpty('当前没有待处理退款')}
        </article>

        <article class="command-panel command-span-6">
          <div class="command-panel-head">
            <div>
              <div class="command-panel-kicker">AUDIT STREAM</div>
              <h3>审计事件流</h3>
            </div>
          </div>
          ${auditStream.length ? auditStream.map((log) => `
            <div class="command-feed-item">
              <div class="command-feed-top">
                <strong>${escapeHtml(log?.user_email || '-')}</strong>
                ${buildCommandCenterStatusBadge(log?.action_taken || 'logged', log?.action_taken === 'blocked' ? 'bad' : (log?.action_taken === 'allowed' ? 'ok' : 'neutral'))}
              </div>
              <div class="command-feed-meta">${escapeHtml(log?.node_name || '-')} · ${escapeHtml(log?.node_location_name || '-')} · ${escapeHtml(log?.ip_address || '-')}</div>
              <div class="command-feed-sub">${escapeHtml(log?.target_domain || log?.target_protocol || '未提供目标')} · ${escapeHtml(formatAnyTimestamp(log?.created_at))}</div>
            </div>
          `).join('') : buildCommandCenterEmpty('当前没有审计事件')}
        </article>
      </section>
    </div>
  `;
}

async function renderAdminCommandCenter() {
  if (!requireAuth()) return;
  const me = await loadMe();
  if (!me?.is_super_admin) {
    setView('<div class="notice error">需要超级管理员权限</div>');
    return;
  }
  const securePath = String(me?.secure_path || '').trim().replace(/^\/+|\/+$/g, '');
  if (!securePath) {
    setView('<div class="notice error">未找到后台安全路径，无法打开独立监控大屏</div>');
    return;
  }

  const target = `/${encodeURIComponent(securePath)}/command-center`;
  setView(html`
    <div class="command-surface">
      <section class="command-hero-panel" style="min-height: calc(100vh - 48px); align-content: center;">
        <div class="command-hero-copy">
          <div class="command-kicker">SUPER ADMIN COMMAND CENTER</div>
          <h2>正在切换到独立监控大屏</h2>
          <p>该页面已经从公共主题壳中独立出去，防止再落到公开概览。若浏览器未自动跳转，请手动打开下方链接。</p>
          <div class="command-hero-badges">
            ${buildCommandCenterStatusBadge(me?.email || 'super-admin', 'ok')}
            ${buildCommandCenterStatusBadge(securePath, 'neutral')}
          </div>
        </div>
        <div class="command-focus-grid">
          <a class="btn primary" href="${target}">打开独立监控大屏</a>
          <a class="btn" href="/${encodeURIComponent(securePath)}">返回完整后台</a>
        </div>
      </section>
    </div>
  `);

  const timer = window.setTimeout(() => {
    window.location.replace(target);
  }, 80);
  registerViewCleanup(() => window.clearTimeout(timer));
}

async function renderAdmin(errorText = '', activeCardId = '') {
  if (!requireAuth()) return;
  const me = await loadMe();
  if (!me?.is_super_admin) {
    setView(`<div class="notice error">需要超级管理员权限</div>`);
    return;
  }

  const unwrap = (payload) => payload?.data ?? payload;
  const safeJson = (val) => {
    try { return JSON.stringify(val ?? null, null, 2); } catch { return String(val); }
  };
  const toNumOrNull = (val) => {
    const raw = String(val ?? '').trim();
    if (!raw) return null;
    const n = Number(raw);
    return Number.isFinite(n) ? n : null;
  };

  let nodePlans = [];
  let nodePlanErr = '';
  let adminNodeOptions = [];
  let adminNodeOptionsErr = '';
  let refunds = [];
  let refundErr = '';
  let groupLimits = [];
  let groupErr = '';
  let apiKeyStats = null;
  let apiKeyErr = '';
  let sponsorProfile = null;
  let sponsorErr = '';
  let adminConfig = {};
  let configErr = '';
  let migratedV2Plans = [];
  let migratedV2PlanErr = '';
  let migratedV2Payments = [];
  let migratedV2PaymentErr = '';
  let migratedV2PaymentMethods = [];
  let migratedV2Notices = [];
  let migratedV2NoticeErr = '';
  let migratedV2Tickets = [];
  let migratedV2TicketErr = '';
  let migratedV2Coupons = [];
  let migratedV2CouponErr = '';
  let migratedV2GiftTemplates = [];
  let migratedV2GiftCodes = [];
  let migratedV2GiftErr = '';
  let migratedV2Plugins = [];
  let migratedV2PluginErr = '';
  let migratedV2SystemStatus = null;
  let migratedV2SystemErr = '';
  let migratedV2TrafficStats = null;
  let migratedV2TrafficErr = '';

  const securePath = String(me?.secure_path || '').trim();
  const buildV2 = (endpoint) => {
    const cleaned = String(endpoint || '').replace(/^\/+/, '');
    if (!securePath) {
      throw new Error('未找到后台路径，无法读取站点安全配置');
    }
    return `/api/v2/${encodeURIComponent(securePath)}/${cleaned}`;
  };

  await Promise.all([
    apiFetch('/api/v1/admin/node-plans')
      .then((res) => { nodePlans = asArray(unwrap(res)); })
      .catch((e) => { nodePlanErr = e.message || '加载失败'; }),
    apiFetch('/api/v1/admin/node-plans/node-options')
      .then((res) => { adminNodeOptions = asArray(unwrap(res)); })
      .catch((e) => {
        adminNodeOptionsErr = e.message || '加载失败';
        adminNodeOptions = [];
      }),
    apiFetch('/api/v1/admin/refunds')
      .then((res) => { refunds = asArray(unwrap(res)); })
      .catch((e) => {
        refundErr = e.message || '加载失败';
        refunds = [];
      }),
    apiFetch('/api/v1/admin/group-limits')
      .then((res) => { groupLimits = asArray(unwrap(res)); })
      .catch((e) => { groupErr = e.message || '加载失败'; }),
    apiFetch('/api/v1/admin/api-keys/stats')
      .then((res) => { apiKeyStats = unwrap(res) || {}; })
      .catch((e) => { apiKeyErr = e.message || '加载失败'; }),
    apiFetch('/api/v1/admin/sponsor-epay')
      .then((res) => { sponsorProfile = unwrap(res) || null; })
      .catch((e) => { sponsorErr = e.message || '加载失败'; }),
    (securePath
      ? apiFetch(buildV2('config/fetch'))
      : Promise.reject(new Error('未找到后台路径，无法读取站点安全配置')))
      .then((res) => { adminConfig = unwrap(res) || {}; })
      .catch((e) => {
        configErr = e.message || '加载失败';
        adminConfig = {};
      }),
    (securePath
      ? apiFetch(buildV2('plan/fetch'))
      : Promise.reject(new Error('未找到后台路径，无法读取订阅套餐')))
      .then((res) => { migratedV2Plans = asArray(unwrap(res)); })
      .catch((e) => {
        migratedV2PlanErr = e.message || '加载失败';
        migratedV2Plans = [];
      }),
    (securePath
      ? apiFetch(buildV2('payment/fetch'))
      : Promise.reject(new Error('未找到后台路径，无法读取支付方式')))
      .then((res) => { migratedV2Payments = asArray(unwrap(res)); })
      .catch((e) => {
        migratedV2PaymentErr = e.message || '加载失败';
        migratedV2Payments = [];
      }),
    (securePath
      ? apiFetch(buildV2('payment/getPaymentMethods'))
      : Promise.resolve({ data: [] }))
      .then((res) => { migratedV2PaymentMethods = asArray(unwrap(res)); })
      .catch(() => { migratedV2PaymentMethods = []; }),
    (securePath
      ? apiFetch(buildV2('notice/fetch'))
      : Promise.reject(new Error('未找到后台路径，无法读取公告')))
      .then((res) => { migratedV2Notices = asArray(unwrap(res)); })
      .catch((e) => {
        migratedV2NoticeErr = e.message || '加载失败';
        migratedV2Notices = [];
      }),
    (securePath
      ? apiFetch(`${buildV2('ticket/fetch')}?current=1&pageSize=20`)
      : Promise.reject(new Error('未找到后台路径，无法读取工单')))
      .then((res) => { migratedV2Tickets = asArray(Array.isArray(res?.data) ? res.data : unwrap(res)); })
      .catch((e) => {
        migratedV2TicketErr = e.message || '加载失败';
        migratedV2Tickets = [];
      }),
    (securePath
      ? apiFetch(`${buildV2('coupon/fetch')}?current=1&pageSize=20`)
      : Promise.reject(new Error('未找到后台路径，无法读取优惠券')))
      .then((res) => { migratedV2Coupons = asArray(Array.isArray(res?.data) ? res.data : unwrap(res)); })
      .catch((e) => {
        migratedV2CouponErr = e.message || '加载失败';
        migratedV2Coupons = [];
      }),
    (securePath
      ? apiFetch(`${buildV2('gift-card/templates')}?per_page=20&page=1`)
      : Promise.reject(new Error('未找到后台路径，无法读取礼品卡模板')))
      .then((res) => {
        migratedV2GiftTemplates = asArray(Array.isArray(res?.data) ? res.data : unwrap(res));
      })
      .catch((e) => {
        migratedV2GiftErr = e.message || '加载失败';
        migratedV2GiftTemplates = [];
      }),
    (securePath
      ? apiFetch(`${buildV2('gift-card/codes')}?per_page=20&page=1`)
      : Promise.resolve({ data: [] }))
      .then((res) => { migratedV2GiftCodes = asArray(Array.isArray(res?.data) ? res.data : unwrap(res)); })
      .catch(() => { migratedV2GiftCodes = []; }),
    (securePath
      ? apiFetch(buildV2('plugin/getPlugins'))
      : Promise.reject(new Error('未找到后台路径，无法读取插件列表')))
      .then((res) => { migratedV2Plugins = asArray(unwrap(res)); })
      .catch((e) => {
        migratedV2PluginErr = e.message || '加载失败';
        migratedV2Plugins = [];
      }),
    (securePath
      ? apiFetch(buildV2('system/getSystemStatus'))
      : Promise.reject(new Error('未找到后台路径，无法读取系统状态')))
      .then((res) => { migratedV2SystemStatus = unwrap(res) || {}; })
      .catch((e) => {
        migratedV2SystemErr = e.message || '加载失败';
        migratedV2SystemStatus = null;
      }),
    (securePath
      ? apiFetch(`${buildV2('traffic-reset/stats')}?days=30`)
      : Promise.reject(new Error('未找到后台路径，无法读取流量重置统计')))
      .then((res) => { migratedV2TrafficStats = unwrap(res) || {}; })
      .catch((e) => {
        migratedV2TrafficErr = e.message || '加载失败';
        migratedV2TrafficStats = null;
      }),
  ]);

  const normalizeList = (value) => {
    if (Array.isArray(value)) return value;
    if (value && typeof value === 'object') return Object.values(value);
    return [];
  };
  const normalizeStringList = (value) => {
    if (Array.isArray(value)) return value.map((v) => String(v));
    if (value && typeof value === 'object') return Object.keys(value);
    return [];
  };

  nodePlans = normalizeList(nodePlans);
  adminNodeOptions = normalizeList(adminNodeOptions);
  refunds = normalizeList(refunds);
  groupLimits = normalizeList(groupLimits);
  migratedV2Plans = normalizeList(migratedV2Plans);
  migratedV2Payments = normalizeList(migratedV2Payments);
  migratedV2PaymentMethods = normalizeStringList(migratedV2PaymentMethods);
  migratedV2Notices = normalizeList(migratedV2Notices);
  migratedV2Tickets = normalizeList(migratedV2Tickets);
  migratedV2Coupons = normalizeList(migratedV2Coupons);
  migratedV2GiftTemplates = normalizeList(migratedV2GiftTemplates);
  migratedV2GiftCodes = normalizeList(migratedV2GiftCodes);
  migratedV2Plugins = normalizeList(migratedV2Plugins);

  const safeCfg = adminConfig?.safe || {};
  const oauthCfg = adminConfig?.oauth || {};
  const siteCfg = adminConfig?.site || {};
  const appCfg = adminConfig?.app || {};
  const subscribeTemplateCfg = adminConfig?.subscribe_template || {};
  const systemCfg = adminConfig?.system || {};
  const activeFrontendTheme = String(siteCfg?.frontend_theme || 'Maintainable');

  setView(html`
    <div class="card admin-command-center-entry" style="margin-bottom:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <div>
          <h2>超级管理员监控大屏</h2>
          <div class="muted">以独立全屏页面打开内部指挥舱，集中查看节点、用户、TCPing、退款、工单与审计流，不做脱敏。</div>
	        </div>
	        <div class="row end">
	          ${me?.secure_path ? `<a class="btn primary" href="/${encodeURIComponent(String(me.secure_path).replace(/^\/+|\/+$/g, ''))}/command-center" target="_blank" rel="noopener">独立打开监控大屏</a>` : ''}
	          ${me?.secure_path ? `<a class="btn" href="/${encodeURIComponent(String(me.secure_path).replace(/^\/+|\/+$/g, ''))}" target="_blank" rel="noopener">打开完整后台</a>` : ''}
	        </div>
	      </div>
	    </div>

    <div class="grid cols-2">
      <div class="card">
        <div class="row" style="justify-content: space-between; align-items: center;">
          <h2>同用户异 IP 限制</h2>
          ${me?.secure_path ? `<a class="btn small" target="_blank" rel="noopener" href="/${encodeURIComponent(String(me.secure_path).replace(/^\/+|\/+$/g, ''))}">打开完整后台</a>` : ''}
        </div>
        ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
        <div class="grid">
          <div class="field"><label>用户 ID</label><input id="adm_uid" type="number" placeholder="例如 123"></div>
          <div class="field"><label>同用户异 IP 上限（单节点，0=按用户组）</label><input id="adm_user_device_limit" type="number" min="0" value="0"></div>
          <div class="field"><label>同用户多 IP 并发上限（0=默认 3）</label><input id="adm_limit" type="number" min="0" value="0"></div>
          <div class="row end">
            <button class="btn" id="adm_load_ip_btn">读取当前值</button>
            <button class="btn primary" id="adm_set_btn">保存这两项限制</button>
          </div>
        </div>
        <div class="muted" style="margin-top:10px;">同用户异 IP 上限：限制同一用户在同一节点可同时使用的不同 IP 数量。</div>
        <div class="muted">同一 IP 跨节点上限：限制同一用户在多个节点重复使用同一个 IP 的并发数量。</div>
        <pre style="margin-top:10px;max-height:220px;overflow:auto;" id="adm_user_limit_detail">输入用户 ID 后点击“读取当前值”。</pre>
      </div>

      <div class="card">
        <div class="row" style="justify-content: space-between; align-items: center;">
          <h2>API Key 系统统计</h2>
          <button class="btn small" id="admApiRefreshStats">刷新</button>
        </div>
        ${apiKeyErr ? `<div class="notice error">${escapeHtml(apiKeyErr)}</div>` : ''}
        <div class="kvs" id="admApiStatsBox">
          ${apiKeyStats && typeof apiKeyStats === 'object'
            ? Object.keys(apiKeyStats).map((k) => html`<div class="k">${k}</div><div class="v">${apiKeyStats[k] ?? '-'}</div>`).join('')
            : '<div class="muted">暂无统计数据。</div>'}
        </div>
        <div class="row end" style="margin-top:12px;">
          <button class="btn" id="admApiBatchGenerate">批量生成缺失 Key</button>
          <button class="btn danger" id="admApiCleanup">清理无效 Key</button>
        </div>
      </div>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <div class="row" style="justify-content: space-between; align-items: center;">
          <h2>登录与安全设置</h2>
          <button class="btn small" id="admReloadConfigBtn">重新读取</button>
        </div>
        ${configErr ? `<div class="notice error">${escapeHtml(configErr)}</div>` : ''}
        <div class="grid">
          <div class="field"><label>后台域名保护</label>
            <select id="sec_safe_mode_enable">
              <option value="1" ${safeCfg.safe_mode_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.safe_mode_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>注册后必须邮箱验证</label>
            <select id="sec_email_verify">
              <option value="1" ${safeCfg.email_verify ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.email_verify ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>注册方式</label>
            <select id="sec_register_mode">
              <option value="all" ${String(safeCfg.register_mode || 'all') === 'all' ? 'selected' : ''}>邮箱 + OAuth</option>
              <option value="email_only" ${String(safeCfg.register_mode || '') === 'email_only' ? 'selected' : ''}>仅邮箱注册</option>
              <option value="oauth_only" ${String(safeCfg.register_mode || '') === 'oauth_only' ? 'selected' : ''}>仅 OAuth 注册</option>
              <option value="closed" ${String(safeCfg.register_mode || '') === 'closed' ? 'selected' : ''}>关闭注册</option>
            </select>
          </div>
          <div class="field"><label>后台入口路径（至少 8 位）</label><input id="sec_secure_path" value="${safeCfg.secure_path || ''}" /></div>
          <div class="field"><label>邮箱登录模式</label>
            <select id="sec_force_oauth2_login">
              <option value="0" ${safeCfg.force_oauth2_login ? '' : 'selected'}>允许邮箱登录（默认）</option>
              <option value="1" ${safeCfg.force_oauth2_login ? 'selected' : ''}>仅超管可邮箱登录（普通用户强制 OAuth2）</option>
            </select>
          </div>
          <div class="field"><label>登录态有效期（天，0=永不过期）</label><input id="sec_login_token_expire_days" type="number" min="0" max="3650" value="${safeCfg.login_token_expire_days ?? 365}" /></div>
          <div class="notice" style="grid-column: 1 / -1;">
            开启后：普通用户将无法使用邮箱密码/邮箱链接登录，必须通过 OAuth2 登录；超级管理员邮箱登录不受影响。
          </div>
          <div class="notice" style="grid-column: 1 / -1;">
            登录态有效期仅影响新登录生成的 Token；设置为 0 表示不设置过期时间（需自行做好风控）。
          </div>
          <div class="field"><label>邮箱后缀白名单</label>
            <select id="sec_email_whitelist_enable">
              <option value="1" ${safeCfg.email_whitelist_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.email_whitelist_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>允许注册的邮箱后缀（逗号分隔）</label><textarea id="sec_email_whitelist_suffix" rows="2" placeholder="例如 qq.com,gmail.com">${escapeHtml(Array.isArray(safeCfg.email_whitelist_suffix) ? safeCfg.email_whitelist_suffix.join(',') : (safeCfg.email_whitelist_suffix || ''))}</textarea></div>
          <div class="field"><label>Gmail 特殊规则限制</label>
            <select id="sec_email_gmail_limit_enable">
              <option value="1" ${safeCfg.email_gmail_limit_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.email_gmail_limit_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>按 IP 限制注册</label>
            <select id="sec_register_limit_by_ip_enable">
              <option value="1" ${safeCfg.register_limit_by_ip_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.register_limit_by_ip_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>注册次数上限</label><input id="sec_register_limit_count" type="number" min="1" value="${safeCfg.register_limit_count ?? 3}" /></div>
          <div class="field"><label>注册统计时长（分钟）</label><input id="sec_register_limit_expire" type="number" min="1" value="${safeCfg.register_limit_expire ?? 60}" /></div>
          <div class="field"><label>密码错误保护</label>
            <select id="sec_password_limit_enable">
              <option value="1" ${safeCfg.password_limit_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.password_limit_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>密码错误上限</label><input id="sec_password_limit_count" type="number" min="1" value="${safeCfg.password_limit_count ?? 5}" /></div>
          <div class="field"><label>密码错误锁定时长（分钟）</label><input id="sec_password_limit_expire" type="number" min="1" value="${safeCfg.password_limit_expire ?? 60}" /></div>
        </div>

        <h3 style="margin-top:12px;">人机验证</h3>
        <div class="grid">
          <div class="field"><label>启用人机验证</label>
            <select id="sec_captcha_enable">
              <option value="1" ${safeCfg.captcha_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.captcha_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>验证服务</label>
            <select id="sec_captcha_type">
              <option value="recaptcha" ${String(safeCfg.captcha_type || 'recaptcha') === 'recaptcha' ? 'selected' : ''}>Google 验证（普通）</option>
              <option value="recaptcha-v3" ${String(safeCfg.captcha_type || '') === 'recaptcha-v3' ? 'selected' : ''}>Google 验证（无感）</option>
              <option value="turnstile" ${String(safeCfg.captcha_type || '') === 'turnstile' ? 'selected' : ''}>Cloudflare Turnstile</option>
            </select>
          </div>
          <div class="notice" id="sec_captcha_provider_hint" style="grid-column: 1 / -1;">
            选择服务商后，只显示当前服务商需要填写的字段。
          </div>
          <div class="field captcha-provider-field" data-captcha-provider="recaptcha"><label>Google 服务端密钥</label><input id="sec_recaptcha_key" value="${safeCfg.recaptcha_key || ''}" /></div>
          <div class="field captcha-provider-field" data-captcha-provider="recaptcha"><label>Google 网页端密钥</label><input id="sec_recaptcha_site_key" value="${safeCfg.recaptcha_site_key || ''}" /></div>
          <div class="field captcha-provider-field" data-captcha-provider="recaptcha-v3"><label>Google 无感服务端密钥</label><input id="sec_recaptcha_v3_secret_key" value="${safeCfg.recaptcha_v3_secret_key || ''}" /></div>
          <div class="field captcha-provider-field" data-captcha-provider="recaptcha-v3"><label>Google 无感网页端密钥</label><input id="sec_recaptcha_v3_site_key" value="${safeCfg.recaptcha_v3_site_key || ''}" /></div>
          <div class="field captcha-provider-field" data-captcha-provider="recaptcha-v3"><label>Google 无感阈值（0-1）</label><input id="sec_recaptcha_v3_score_threshold" type="number" min="0" max="1" step="0.01" value="${safeCfg.recaptcha_v3_score_threshold ?? 0.5}" /></div>
          <div class="field captcha-provider-field" data-captcha-provider="turnstile"><label>站点密钥</label><input id="sec_turnstile_site_key" value="${safeCfg.turnstile_site_key || ''}" /></div>
          <div class="field captcha-provider-field" data-captcha-provider="turnstile"><label>密钥</label><input id="sec_turnstile_secret_key" value="${safeCfg.turnstile_secret_key || ''}" /></div>
        </div>

        <h3 style="margin-top:12px;">防刷挑战（PoW）</h3>
        <div class="grid">
          <div class="field"><label>启用防刷挑战</label>
            <select id="sec_pow_enable">
              <option value="1" ${safeCfg.pow_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.pow_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>自动调节难度</label>
            <select id="sec_pow_auto_scale_enable">
              <option value="1" ${safeCfg.pow_auto_scale_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.pow_auto_scale_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>挑战难度（1-8）</label><input id="sec_pow_difficulty" type="number" min="1" max="8" value="${safeCfg.pow_difficulty ?? 4}" /></div>
          <div class="field"><label>挑战有效期（秒）</label><input id="sec_pow_ttl" type="number" min="30" max="600" value="${safeCfg.pow_ttl ?? 120}" /></div>
          <div class="field"><label>自动调节上限</label><input id="sec_pow_auto_max_difficulty" type="number" min="1" max="8" value="${safeCfg.pow_auto_max_difficulty ?? 7}" /></div>
          <div class="field"><label>当前生效难度</label><input id="sec_pow_effective_difficulty" value="${safeCfg.pow_effective_difficulty ?? safeCfg.pow_difficulty ?? 4}" disabled /></div>
          <div class="field"><label>挑战盐值（可空）</label><input id="sec_pow_seed_salt" value="${safeCfg.pow_seed_salt || ''}" /></div>
          <div class="field"><label>挑战基础值（可空）</label><input id="sec_pow_base_value" value="${safeCfg.pow_base_value || ''}" /></div>
          <div class="field"><label>校验设备指纹</label>
            <select id="sec_pow_require_ja3">
              <option value="1" ${safeCfg.pow_require_ja3 ? 'selected' : ''}>开启</option>
              <option value="0" ${safeCfg.pow_require_ja3 ? '' : 'selected'}>关闭</option>
            </select>
          </div>
        </div>

        <div class="row end" style="margin-top:12px;">
          <button class="btn primary" id="admSaveSecurityBtn">保存安全设置</button>
        </div>
      </div>

      <div class="card">
        <h2>OAuth2 登录设置</h2>
        ${configErr ? `<div class="notice error">${escapeHtml(configErr)}</div>` : ''}
        <div class="grid">
          <div class="field"><label>启用 Linux DO 登录</label>
            <select id="oauth_linux_do_enable">
              <option value="1" ${oauthCfg.oauth_linux_do_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${oauthCfg.oauth_linux_do_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>Client ID</label><input id="oauth_linux_do_client_id" value="${oauthCfg.oauth_linux_do_client_id || ''}" placeholder="从 Linux DO 应用后台复制" /></div>
          <div class="field"><label>Client Secret</label><input id="oauth_linux_do_client_secret" type="password" value="${oauthCfg.oauth_linux_do_client_secret || ''}" placeholder="从 Linux DO 应用后台复制" /></div>
          <div class="field"><label>回调地址</label><input id="oauth_linux_do_redirect_uri" value="${oauthCfg.oauth_linux_do_redirect_uri || ''}" /></div>
        </div>
        <div class="row end" style="margin-top:12px;">
          <button class="btn" id="admOauthCallbackBtn">填入推荐回调地址</button>
          <button class="btn primary" id="admSaveOauthBtn">保存 OAuth2 设置</button>
        </div>
        <div class="muted" style="margin-top:10px;">
          登录入口固定为 <code>/api/v1/passport/oauth2/linux-do/redirect</code>。
        </div>
        <div class="muted">
          当前站点地址：<code>${escapeHtml(siteCfg.app_url || window.__APP__?.baseUrl || location.origin)}</code>
        </div>

        <h3 style="margin-top:12px;">注册与订阅入口</h3>
        <div class="grid">
          <div class="field"><label>订阅完整域名（可留空，支持多行）</label><textarea id="site_subscribe_url" rows="2" placeholder="https://sub.example.com">${escapeHtml(siteCfg.subscribe_url || '')}</textarea></div>
          <div class="field"><label>订阅根域名（每行一个，随机子域名）</label><textarea id="site_subscribe_root_domains" rows="2" placeholder="example.com&#10;sub.example.net">${escapeHtml(siteCfg.subscribe_root_domains || '')}</textarea></div>
        </div>
        <div class="row end" style="margin-top:12px;">
          <button class="btn primary" id="admSaveSiteRoutingBtn">保存注册与订阅入口</button>
        </div>
      </div>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <h2>客户端版本与下载地址</h2>
        <div class="muted">用于前端“客户端下载”展示与版本提醒。</div>
        <div class="grid cols-2" style="margin-top:10px;">
          <div class="field"><label>Windows 版本号</label><input id="app_windows_version" value="${escapeHtml(appCfg.windows_version || '')}" placeholder="例如 2.1.0"></div>
          <div class="field"><label>Windows 下载地址</label><input id="app_windows_download_url" value="${escapeHtml(appCfg.windows_download_url || '')}" placeholder="https://..."></div>
          <div class="field"><label>macOS 版本号</label><input id="app_macos_version" value="${escapeHtml(appCfg.macos_version || '')}" placeholder="例如 2.1.0"></div>
          <div class="field"><label>macOS 下载地址</label><input id="app_macos_download_url" value="${escapeHtml(appCfg.macos_download_url || '')}" placeholder="https://..."></div>
          <div class="field"><label>Android 版本号</label><input id="app_android_version" value="${escapeHtml(appCfg.android_version || '')}" placeholder="例如 2.1.0"></div>
          <div class="field"><label>Android 下载地址</label><input id="app_android_download_url" value="${escapeHtml(appCfg.android_download_url || '')}" placeholder="https://..."></div>
        </div>
        <div class="row end" style="margin-top:12px;">
          <button class="btn primary" id="admSaveAppClientBtn">保存客户端版本设置</button>
        </div>
      </div>

      <div class="card">
        <h2>记录保留与订阅凭据</h2>
        <div class="muted">用于控制监控、流量记录的保留时长，以及是否每天轮换下发订阅凭据。订阅链接本身不会变化。</div>
        <div class="grid cols-2" style="margin-top:10px;">
          <div class="field"><label>每日轮换下发订阅凭据</label>
            <select id="sys_rotate_subscription_credentials_daily">
              <option value="1" ${systemCfg.rotate_subscription_credentials_daily ? 'selected' : ''}>开启</option>
              <option value="0" ${systemCfg.rotate_subscription_credentials_daily ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>启用争议退款投票</label>
            <select id="sys_refund_dispute_enable">
              <option value="1" ${systemCfg.refund_dispute_enable ? 'selected' : ''}>开启</option>
              <option value="0" ${systemCfg.refund_dispute_enable ? '' : 'selected'}>关闭</option>
            </select>
          </div>
          <div class="field"><label>节点流量记录保留天数</label><input id="sys_node_traffic_records_retention_days" type="number" min="1" max="365" value="${systemCfg.node_traffic_records_retention_days ?? 7}"></div>
          <div class="field"><label>用户流量记录保留天数</label><input id="sys_user_traffic_usage_logs_retention_days" type="number" min="1" max="365" value="${systemCfg.user_traffic_usage_logs_retention_days ?? 7}"></div>
          <div class="field"><label>TCPing 采样保留天数</label><input id="sys_tcping_samples_retention_days" type="number" min="1" max="365" value="${systemCfg.tcping_samples_retention_days ?? 7}"></div>
          <div class="field"><label>TCPing 告警保留天数</label><input id="sys_tcping_alerts_retention_days" type="number" min="1" max="365" value="${systemCfg.tcping_alerts_retention_days ?? 7}"></div>
          <div class="field"><label>审计记录保留天数</label><input id="sys_audit_logs_retention_days" type="number" min="1" max="365" value="${systemCfg.audit_logs_retention_days ?? 7}"></div>
        </div>
        <div class="row end" style="margin-top:12px;">
          <button class="btn primary" id="admSaveSystemRetentionBtn">保存系统保留策略</button>
        </div>
      </div>

      <div class="card">
        <h2>代理订阅模板</h2>
        <div class="muted">模板编辑已拆分为独立全屏页面，便于查看长内容和细节差异。</div>
        <div class="grid cols-2" style="margin-top:10px;">
          ${SUBSCRIBE_TEMPLATE_EDITORS.map((item) => `
            <a class="btn" href="#/admin/template/${item.key}">
              编辑 ${escapeHtml(item.label)} 模板（${item.format.toUpperCase()}）
            </a>
          `).join('')}
        </div>
      </div>
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>已迁移模块（原安全路径）</h2>
        <button class="btn small" id="admV2ReloadBtn">刷新这些模块</button>
      </div>
      <div class="kvs" style="margin-top:10px;">
        <div class="k">订阅套餐</div><div class="v">${migratedV2Plans.length}</div>
        <div class="k">支付方式</div><div class="v">${migratedV2Payments.length}</div>
        <div class="k">公告</div><div class="v">${migratedV2Notices.length}</div>
        <div class="k">工单</div><div class="v">${migratedV2Tickets.length}</div>
        <div class="k">优惠券</div><div class="v">${migratedV2Coupons.length}</div>
        <div class="k">礼品卡模板</div><div class="v">${migratedV2GiftTemplates.length}</div>
        <div class="k">插件</div><div class="v">${migratedV2Plugins.length}</div>
      </div>
      <div class="muted" style="margin-top:10px;">仅超级管理员页面可见，普通用户和普通管理员不会出现此模块。</div>
      <div class="muted">核心安全点：所有操作仍走后端权限中间件，前端仅做可视化入口。</div>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <h2>固定前端主题与订阅套餐</h2>
        ${migratedV2PlanErr ? `<div class="notice error">${escapeHtml(migratedV2PlanErr)}</div>` : ''}
        <div class="grid">
          <div class="field">
            <label>前端主题（固定）</label>
            <input value="${escapeHtml(activeFrontendTheme)}" disabled />
          </div>
        </div>
        <div class="muted" style="margin-top:10px;">主题切换接口已移除，当前站点固定使用该主题作为默认主题。</div>
        <div class="grid cols-2" style="margin-top:10px;">
          <div class="field"><label>套餐名</label><input id="v2_plan_name" placeholder="例如 入门套餐" /></div>
          <div class="field"><label>流量额度(GB)</label><input id="v2_plan_transfer_gb" type="number" min="1" value="100" /></div>
          <div class="field"><label>月付(元)</label><input id="v2_plan_price_month" type="number" min="0" step="0.01" value="10" /></div>
          <div class="field"><label>季付(元)</label><input id="v2_plan_price_quarter" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>半年付(元)</label><input id="v2_plan_price_half_year" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>年付(元)</label><input id="v2_plan_price_year" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>两年付(元)</label><input id="v2_plan_price_two_year" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>三年付(元)</label><input id="v2_plan_price_three_year" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>一次性(元)</label><input id="v2_plan_price_onetime" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>重置流量包(元)</label><input id="v2_plan_price_reset" type="number" min="0" step="0.01" /></div>
          <div class="field"><label>速度限制(Mbps)</label><input id="v2_plan_speed" type="number" min="0" value="0" /></div>
          <div class="field"><label>设备限制</label><input id="v2_plan_device" type="number" min="0" value="0" /></div>
          <div class="field"><label>容量限制(0不限)</label><input id="v2_plan_capacity" type="number" min="0" value="0" /></div>
          <div class="field"><label>说明</label><input id="v2_plan_content" placeholder="可空" /></div>
          <div class="field"><label>权限组 ID</label><input id="v2_plan_group_id" type="number" min="0" value="0" /></div>
          <div class="field"><label>显示</label><select id="v2_plan_show"><option value="1" selected>是</option><option value="0">否</option></select></div>
          <div class="field"><label>售卖</label><select id="v2_plan_sell"><option value="1" selected>是</option><option value="0">否</option></select></div>
          <div class="field"><label>续费</label><select id="v2_plan_renew"><option value="1" selected>是</option><option value="0">否</option></select></div>
          <div class="field"><label>排序(sort)</label><input id="v2_plan_sort" type="number" min="0" value="0" /></div>
          <div class="field"><label>重置方式</label>
            <select id="v2_plan_reset_method">
              <option value="0" selected>每月1号</option>
              <option value="1">按月重置</option>
              <option value="2">不重置</option>
              <option value="3">每年1月1号</option>
              <option value="4">按年重置</option>
            </select>
          </div>
          <div class="field"><label>标签（逗号）</label><input id="v2_plan_tags" placeholder="例如 新手,推荐" /></div>
        </div>
        <div class="row end" style="margin-top:10px;">
          <button class="btn primary" id="v2PlanCreateBtn">新建套餐</button>
        </div>
        <table class="table" style="margin-top:10px;">
          <thead><tr><th>ID</th><th>名称</th><th>流量</th><th>显示/售卖/续费</th><th></th></tr></thead>
          <tbody>
            ${migratedV2Plans.slice(0, 20).map((p) => html`
              <tr>
                <td>${p.id}</td>
                <td>${escapeHtml(p.name || '-')}</td>
                <td class="muted">${p.transfer_enable || 0} GB</td>
                <td class="muted">${p.show ? '开' : '关'} / ${p.sell ? '开' : '关'} / ${p.renew ? '开' : '关'}</td>
                <td class="row end">
                  <button class="btn small" data-v2-plan-toggle="${p.id}" data-toggle-key="show">显示</button>
                  <button class="btn small" data-v2-plan-toggle="${p.id}" data-toggle-key="sell">售卖</button>
                  <button class="btn small" data-v2-plan-toggle="${p.id}" data-toggle-key="renew">续费</button>
                  <button class="btn small danger" data-v2-plan-drop="${p.id}">删除</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>

      <div class="card">
        <h2>支付与公告</h2>
        ${(migratedV2PaymentErr || migratedV2NoticeErr) ? `<div class="notice error">${escapeHtml(migratedV2PaymentErr || migratedV2NoticeErr)}</div>` : ''}
        <h3>新增支付方式</h3>
        <div class="grid cols-2">
          <div class="field"><label>显示名称</label><input id="v2_payment_name" placeholder="例如 支付宝" /></div>
          <div class="field"><label>网关类型</label>
            <select id="v2_payment_method">
              ${migratedV2PaymentMethods.map((m) => `<option value="${m}">${m}</option>`).join('')}
            </select>
          </div>
          <div class="field"><label>图标（可空）</label><input id="v2_payment_icon" placeholder="例如 alipay" /></div>
          <div class="field"><label>回调域名（可空）</label><input id="v2_payment_notify_domain" placeholder="https://pay.example.com" /></div>
          <div class="field" style="grid-column: 1 / -1;"><label>配置 JSON</label><textarea id="v2_payment_config" rows="3" placeholder='例如 {"pid":"123","key":"xxx"}'></textarea></div>
        </div>
        <div class="row end" style="margin-top:10px;">
          <button class="btn primary" id="v2PaymentCreateBtn">新增支付方式</button>
        </div>
        <table class="table" style="margin-top:10px;">
          <thead><tr><th>ID</th><th>名称</th><th>类型</th><th>启用</th><th></th></tr></thead>
          <tbody>
            ${migratedV2Payments.slice(0, 20).map((p) => html`
              <tr>
                <td>${p.id}</td>
                <td>${escapeHtml(p.name || '-')}</td>
                <td class="muted">${escapeHtml(p.payment || '-')}</td>
                <td class="muted">${p.enable ? '是' : '否'}</td>
                <td class="row end">
                  <button class="btn small" data-v2-payment-toggle="${p.id}">${p.enable ? '禁用' : '启用'}</button>
                  <button class="btn small danger" data-v2-payment-drop="${p.id}">删除</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>

        <h3 style="margin-top:12px;">发布公告</h3>
        <div class="grid cols-2">
          <div class="field"><label>标题</label><input id="v2_notice_title" placeholder="公告标题" /></div>
          <div class="field"><label>标签（逗号分隔）</label><input id="v2_notice_tags" placeholder="系统,维护" /></div>
          <div class="field" style="grid-column: 1 / -1;"><label>内容</label><textarea id="v2_notice_content" rows="3"></textarea></div>
        </div>
        <div class="row end" style="margin-top:10px;">
          <button class="btn primary" id="v2NoticeCreateBtn">发布公告</button>
        </div>
        <table class="table" style="margin-top:10px;">
          <thead><tr><th>ID</th><th>标题</th><th>显示</th><th></th></tr></thead>
          <tbody>
            ${migratedV2Notices.slice(0, 20).map((n) => html`
              <tr>
                <td>${n.id}</td>
                <td>${escapeHtml(n.title || '-')}</td>
                <td class="muted">${n.show ? '是' : '否'}</td>
                <td class="row end">
                  <button class="btn small" data-v2-notice-toggle="${n.id}">显示开关</button>
                  <button class="btn small danger" data-v2-notice-drop="${n.id}">删除</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <h2>工单与优惠券</h2>
        ${(migratedV2TicketErr || migratedV2CouponErr) ? `<div class="notice error">${escapeHtml(migratedV2TicketErr || migratedV2CouponErr)}</div>` : ''}
        <h3>工单处理</h3>
        <table class="table">
          <thead><tr><th>ID</th><th>用户</th><th>主题</th><th>状态</th><th></th></tr></thead>
          <tbody>
            ${migratedV2Tickets.slice(0, 20).map((t) => html`
              <tr>
                <td>${t.id}</td>
                <td class="muted">${escapeHtml(t?.user?.email || '-')}</td>
                <td>${escapeHtml(t.subject || '-')}</td>
                <td class="muted">${escapeHtml(t.status || '-')}</td>
                <td class="row end">
                  <button class="btn small" data-v2-ticket-detail="${t.id}">查看</button>
                  <button class="btn small danger" data-v2-ticket-close="${t.id}">关闭</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
        <div class="grid cols-2" style="margin-top:10px;">
          <div class="field"><label>回复工单 ID</label><input id="v2_ticket_reply_id" type="number" min="1" /></div>
          <div class="field"><label>回复内容</label><input id="v2_ticket_reply_message" placeholder="请输入回复" /></div>
        </div>
        <div class="row end" style="margin-top:8px;">
          <button class="btn primary" id="v2TicketReplyBtn">发送回复</button>
        </div>
        <pre style="margin-top:10px;max-height:240px;overflow:auto;" id="v2_ticket_out">工单详情输出</pre>

        <h3 style="margin-top:12px;">新增优惠券</h3>
        <div class="grid cols-2">
          <div class="field"><label>名称</label><input id="v2_coupon_name" placeholder="活动券" /></div>
          <div class="field"><label>类型</label>
            <select id="v2_coupon_type">
              <option value="1">减金额（分）</option>
              <option value="2">打折（百分比）</option>
            </select>
          </div>
          <div class="field"><label>值</label><input id="v2_coupon_value" type="number" min="1" value="100" /></div>
          <div class="field"><label>总可用次数</label><input id="v2_coupon_limit_use" type="number" min="0" value="0" /></div>
          <div class="field"><label>每用户可用次数</label><input id="v2_coupon_limit_user" type="number" min="0" value="0" /></div>
          <div class="field"><label>开始时间</label><input id="v2_coupon_start" type="datetime-local" /></div>
          <div class="field"><label>结束时间</label><input id="v2_coupon_end" type="datetime-local" /></div>
          <div class="field"><label>限制套餐 ID（逗号）</label><input id="v2_coupon_plan_ids" placeholder="1,2,3" /></div>
        </div>
        <div class="row end" style="margin-top:8px;">
          <button class="btn primary" id="v2CouponCreateBtn">创建优惠券</button>
        </div>
        <table class="table" style="margin-top:10px;">
          <thead><tr><th>ID</th><th>名称</th><th>券码</th><th>显示</th><th></th></tr></thead>
          <tbody>
            ${migratedV2Coupons.slice(0, 20).map((c) => html`
              <tr>
                <td>${c.id}</td>
                <td>${escapeHtml(c.name || '-')}</td>
                <td class="muted">${escapeHtml(c.code || '-')}</td>
                <td class="muted">${c.show ? '是' : '否'}</td>
                <td class="row end">
                  <button class="btn small" data-v2-coupon-toggle="${c.id}">显示开关</button>
                  <button class="btn small danger" data-v2-coupon-drop="${c.id}">删除</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>

      <div class="card">
        <h2>礼品卡与插件</h2>
        ${(migratedV2GiftErr || migratedV2PluginErr) ? `<div class="notice error">${escapeHtml(migratedV2GiftErr || migratedV2PluginErr)}</div>` : ''}
        <h3>批量生成礼品卡兑换码</h3>
        <div class="grid cols-2">
          <div class="field">
            <label>模板</label>
            <select id="v2_gift_template_id">
              ${migratedV2GiftTemplates.map((t) => `<option value="${t.id}">${escapeHtml(t.name)} (#${t.id})</option>`).join('')}
            </select>
          </div>
          <div class="field"><label>生成数量</label><input id="v2_gift_count" type="number" min="1" max="10000" value="10" /></div>
          <div class="field"><label>前缀</label><input id="v2_gift_prefix" value="GC" /></div>
          <div class="field"><label>每码可用次数</label><input id="v2_gift_max_usage" type="number" min="1" value="1" /></div>
        </div>
        <div class="row end" style="margin-top:8px;">
          <button class="btn primary" id="v2GiftGenerateBtn">生成兑换码</button>
        </div>
        <table class="table" style="margin-top:10px;">
          <thead><tr><th>ID</th><th>模板</th><th>兑换码</th><th>状态</th><th></th></tr></thead>
          <tbody>
            ${migratedV2GiftCodes.slice(0, 20).map((g) => html`
              <tr>
                <td>${g.id}</td>
                <td class="muted">${escapeHtml(g.template_name || g.template_id || '-')}</td>
                <td>${escapeHtml(g.code || '-')}</td>
                <td class="muted">${escapeHtml(g.status_name || g.status || '-')}</td>
                <td class="row end">
                  <button class="btn small" data-v2-gift-toggle="${g.id}" data-v2-gift-action="${String(g.status_name || '').includes('禁用') ? 'enable' : 'disable'}">${String(g.status_name || '').includes('禁用') ? '启用' : '禁用'}</button>
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>

        <h3 style="margin-top:12px;">插件管理</h3>
        <table class="table">
          <thead><tr><th>插件</th><th>状态</th><th></th></tr></thead>
          <tbody>
            ${migratedV2Plugins.slice(0, 30).map((p) => html`
              <tr>
                <td>${escapeHtml(p.name || p.code)}</td>
                <td class="muted">${p.is_installed ? (p.is_enabled ? '已启用' : '已安装') : '未安装'}</td>
                <td class="row end">
                  ${p.is_installed ? `<button class="btn small" data-v2-plugin-act="${p.is_enabled ? 'disable' : 'enable'}" data-v2-plugin-code="${p.code}">${p.is_enabled ? '禁用' : '启用'}</button>` : `<button class="btn small" data-v2-plugin-act="install" data-v2-plugin-code="${p.code}">安装</button>`}
                  ${p.need_upgrade ? `<button class="btn small" data-v2-plugin-act="upgrade" data-v2-plugin-code="${p.code}">升级</button>` : ''}
                  ${p.is_installed ? `<button class="btn small danger" data-v2-plugin-act="uninstall" data-v2-plugin-code="${p.code}">卸载</button>` : ''}
                </td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <h2>系统状态与日志</h2>
        ${migratedV2SystemErr ? `<div class="notice error">${escapeHtml(migratedV2SystemErr)}</div>` : ''}
        <div class="kvs">
          <div class="k">定时任务</div><div class="v">${migratedV2SystemStatus?.schedule ? '正常' : '异常'}</div>
          <div class="k">队列</div><div class="v">${migratedV2SystemStatus?.horizon ? '正常' : '异常'}</div>
          <div class="k">日志总数</div><div class="v">${migratedV2SystemStatus?.logs?.total ?? '-'}</div>
          <div class="k">错误日志</div><div class="v">${migratedV2SystemStatus?.logs?.error ?? '-'}</div>
        </div>
        <div class="grid cols-2" style="margin-top:10px;">
          <div class="field"><label>清理多少天前日志</label><input id="v2_log_days" type="number" min="0" value="30" /></div>
          <div class="field"><label>日志级别</label>
            <select id="v2_log_level">
              <option value="all">全部</option>
              <option value="info">info</option>
              <option value="warning">warning</option>
              <option value="error">error</option>
            </select>
          </div>
          <div class="field"><label>单次数量</label><input id="v2_log_limit" type="number" min="100" max="10000" value="1000" /></div>
        </div>
        <div class="row end" style="margin-top:8px;">
          <button class="btn" id="v2SystemLogsBtn">读取最近日志</button>
          <button class="btn danger" id="v2SystemClearBtn">执行日志清理</button>
        </div>
        <pre style="margin-top:10px;max-height:240px;overflow:auto;" id="v2_system_out">系统日志输出</pre>
      </div>

      <div class="card">
        <h2>流量重置管理</h2>
        ${migratedV2TrafficErr ? `<div class="notice error">${escapeHtml(migratedV2TrafficErr)}</div>` : ''}
        <div class="kvs">
          <div class="k">近30天总重置</div><div class="v">${migratedV2TrafficStats?.total_resets ?? '-'}</div>
          <div class="k">自动重置</div><div class="v">${migratedV2TrafficStats?.auto_resets ?? '-'}</div>
          <div class="k">手动重置</div><div class="v">${migratedV2TrafficStats?.manual_resets ?? '-'}</div>
        </div>
        <div class="grid cols-2" style="margin-top:10px;">
          <div class="field"><label>用户 ID</label><input id="v2_traffic_user_id" type="number" min="1" placeholder="123" /></div>
          <div class="field"><label>备注（可空）</label><input id="v2_traffic_reason" placeholder="例如 手动修复" /></div>
        </div>
        <div class="row end" style="margin-top:8px;">
          <button class="btn" id="v2TrafficHistoryBtn">查看用户历史</button>
          <button class="btn primary" id="v2TrafficResetBtn">重置用户流量</button>
        </div>
        <pre style="margin-top:10px;max-height:240px;overflow:auto;" id="v2_traffic_out">流量重置输出</pre>
      </div>
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>用户组限制（trust_level 0-4）</h2>
        <div class="row end">
          <button class="btn" id="admGroupDefaults">应用默认模板</button>
          <button class="btn" id="admGroupRefresh">刷新</button>
        </div>
      </div>
      ${groupErr ? `<div class="notice error">${escapeHtml(groupErr)}</div>` : ''}
      <table class="table" style="margin-top:10px;">
        <thead>
          <tr><th>等级</th><th>名称</th><th>上行</th><th>下行</th><th>设备</th><th>连接</th><th></th></tr>
        </thead>
        <tbody>
          ${groupLimits.length ? groupLimits.map((g) => html`
            <tr>
              <td>${g.trust_level}</td>
              <td class="muted">${escapeHtml(g.trust_level_name || '-')}</td>
              <td><input id="gl_up_${g.trust_level}" type="number" value="${g.speed_limit_up ?? 0}" /></td>
              <td><input id="gl_down_${g.trust_level}" type="number" value="${g.speed_limit_down ?? 0}" /></td>
              <td><input id="gl_dev_${g.trust_level}" type="number" value="${g.device_limit ?? 0}" /></td>
              <td><input id="gl_conn_${g.trust_level}" type="number" value="${g.connection_limit ?? 0}" /></td>
              <td class="row end">
                <button class="btn small primary" data-gl-save="${g.trust_level}">保存</button>
                <button class="btn small danger" data-gl-del="${g.trust_level}">删除</button>
              </td>
            </tr>
          `).join('') : '<tr><td colspan="7" class="muted">暂无分组限制配置。</td></tr>'}
        </tbody>
      </table>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <div class="row" style="justify-content: space-between; align-items: center;">
          <h2>用户个人限制</h2>
          <button class="btn small" id="admIndLoadBtn">读取</button>
        </div>
        <div class="grid">
          <div class="field"><label>用户 ID</label><input id="adm_ind_uid" type="number" placeholder="123"></div>
          <div class="field"><label>上行限制</label><input id="adm_ind_up" type="number" placeholder="可空"></div>
          <div class="field"><label>下行限制</label><input id="adm_ind_down" type="number" placeholder="可空"></div>
          <div class="field"><label>设备数限制</label><input id="adm_ind_dev" type="number" placeholder="可空"></div>
          <div class="field"><label>连接数限制</label><input id="adm_ind_conn" type="number" placeholder="可空"></div>
          <div class="row end">
            <button class="btn primary" id="admIndSaveBtn">保存个人限制</button>
            <button class="btn danger" id="admIndDeleteBtn">删除个人限制</button>
          </div>
        </div>
        <pre style="margin-top:10px;max-height:260px;overflow:auto;" id="adm_individual_detail">点击“读取”加载详情。</pre>
      </div>

      ${me?.is_super_admin ? html`
        <div class="card">
          <h2>赞助收款配置（超管）</h2>
          ${sponsorErr ? `<div class="notice error">${escapeHtml(sponsorErr)}</div>` : ''}
          <div class="grid">
            <div class="field"><label>网关 URL</label><input id="sp_url" value="${escapeHtml(sponsorProfile?.url || 'https://credit.linux.do/epay')}"></div>
            <div class="field"><label>提交路径</label><input id="sp_submit" value="${escapeHtml(sponsorProfile?.submit_path || '/pay/submit.php')}"></div>
            <div class="field"><label>商户号 PID</label><input id="sp_pid" value="${escapeHtml(sponsorProfile?.pid || '')}"></div>
            <div class="field"><label>商户密钥</label><input id="sp_key" type="password" placeholder="${sponsorProfile?.has_key ? '已设置，重新输入可覆盖' : ''}"></div>
            <div class="field"><label>提交方式</label>
              <select id="sp_post">
                <option value="1" ${(sponsorProfile?.use_post ?? true) ? 'selected' : ''}>POST</option>
                <option value="0" ${(sponsorProfile?.use_post ?? true) ? '' : 'selected'}>GET</option>
              </select>
            </div>
            <div class="field"><label>站点名称（可选）</label><input id="sp_site" value="${escapeHtml(sponsorProfile?.sitename || '')}"></div>
            <div class="field"><label>设备标识（可选）</label><input id="sp_device" value="${escapeHtml(sponsorProfile?.device || '')}"></div>
            <div class="row end">
              <button class="btn primary" id="admSponsorSaveBtn">保存赞助收款</button>
            </div>
          </div>
        </div>
      ` : ''}
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>API Key 搜索与用户操作</h2>
      </div>
      <div class="grid cols-3" style="margin-top:10px;">
        <div class="field"><label>关键词（邮箱/用户名/Key）</label><input id="adm_api_query" placeholder="至少 3 位"></div>
        <div class="field"><label>每页数量</label><input id="adm_api_per_page" type="number" value="20"></div>
        <div class="row end">
          <button class="btn primary" id="admApiSearchBtn">搜索</button>
        </div>
      </div>
      <table class="table" style="margin-top:10px;">
        <thead><tr><th>ID</th><th>邮箱</th><th>用户名</th><th>Key 前缀</th><th></th></tr></thead>
        <tbody id="adm_api_search_tbody">
          <tr><td colspan="5" class="muted">请输入关键词后搜索。</td></tr>
        </tbody>
      </table>
      <pre style="margin-top:10px;max-height:260px;overflow:auto;" id="adm_api_detail">API Key 详情输出</pre>
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>节点套餐（全局）</h2>
        <button class="btn primary" id="admNewNodePlan">新建</button>
      </div>
      ${(nodePlanErr || adminNodeOptionsErr) ? `<div class="notice error">${escapeHtml(nodePlanErr || adminNodeOptionsErr)}</div>` : ''}
      <div class="muted" style="margin-top:10px;">超管可管理全部用户发布的节点套餐：编辑、上架开关、售卖开关、续费开关、删除。</div>
      <table class="table" style="margin-top:10px;">
        <thead><tr><th>ID</th><th>名称</th><th>owner</th><th>节点</th><th>可见方式</th><th>试用</th><th>付费额度(GB)</th><th>专属购买链接</th><th>显示/售卖/续费</th><th></th></tr></thead>
        <tbody>
          ${nodePlans.slice(0, 60).map((p) => html`
            <tr>
              <td>${p.id}</td>
              <td>${escapeHtml(String(p.name || ''))}</td>
              <td class="muted">${p.owner_user_id || '-'}</td>
              <td class="muted">${asArray(p.node_ids).join(',')}</td>
              <td class="muted">
                ${normalizePlanVisibilityScope(p.visibility_scope) === 'link_only'
                  ? '<span class="pill">仅专属链接</span>'
                  : (normalizePlanVisibilityScope(p.visibility_scope) === 'assigned_only'
                    ? '<span class="pill">仅指定用户</span>'
                    : `<span class="pill ${p.show ? 'ok' : ''}">${p.show ? '公开展示' : '不公开展示'}</span>`)}
              </td>
              <td>${p.allow_trial || hasPositiveTrialQuota(p.free_quota_gb_by_trust_level) ? '<span class="pill ok">允许</span>' : '<span class="pill">不允许</span>'}</td>
              <td class="muted">${p.is_unlimited_traffic ? '<span class="pill ok">无限</span>' : (Number(p.paid_quota_gb ?? p.transfer_enable ?? 0) || 0)}</td>
              <td>
                ${p.share_token
                  ? `<div class="row">
                      <input readonly value="${escapeHtml(buildNodePlanShareLink(p.share_token))}">
                      <button class="btn small" data-adm-copy-plan-link="${escapeHtml(buildNodePlanShareLink(p.share_token))}">复制</button>
                    </div>`
                  : '<span class="muted">未启用</span>'}
              </td>
              <td class="muted">${p.show ? '开' : '关'} / ${p.sell ? '开' : '关'} / ${p.renew ? '开' : '关'}</td>
              <td class="row end">
                <button class="btn small" data-adm-toggle="${p.id}" data-adm-toggle-key="show">${p.show ? '下架' : '上架'}</button>
                <button class="btn small" data-adm-toggle="${p.id}" data-adm-toggle-key="sell">${p.sell ? '停售' : '开售'}</button>
                <button class="btn small" data-adm-toggle="${p.id}" data-adm-toggle-key="renew">${p.renew ? '停续费' : '开续费'}</button>
                <button class="btn small" data-adm-edit="${p.id}">编辑</button>
                <button class="btn small danger" data-adm-del="${p.id}">删除</button>
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>

    <div class="card" style="margin-top:12px;">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>退款（全局）</h2>
        <button class="btn" id="admRefundFinalize">结算到期投票</button>
      </div>
      ${refundErr ? `<div class="notice error">${escapeHtml(refundErr)}</div>` : ''}
      <table class="table" style="margin-top:10px;">
        <thead><tr><th>ID</th><th>订单</th><th>用户</th><th>状态</th><th>退回/收费(分)</th><th></th></tr></thead>
        <tbody>
          ${refunds.slice(0, 60).map((r) => html`
            <tr>
              <td>${r.id}</td>
              <td class="muted">${r.trade_no}</td>
              <td class="muted">${r.user_id}</td>
              <td><span class="pill">${r.status}</span></td>
              <td class="muted">${r.refund_amount ?? '-'} / ${r.charged_amount ?? '-'}</td>
              <td class="row end">
                <button class="btn small primary" data-adm-refund-approve="${r.id}">同意</button>
                <button class="btn small danger" data-adm-refund-deny="${r.id}">拒绝</button>
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
      <div class="muted" style="margin-top:10px;">超管可覆盖裁定；投票到期也会由定时任务自动结算。</div>
    </div>
  `);

  const adminSidebar = mountAdminSidebar(activeCardId);
  const rerender = async (msg = '') => {
    await renderAdmin(msg, adminSidebar.getCurrent());
  };

  const readBoolSelect = (id) => String(qs(`#${id}`)?.value || '0') === '1';
  const readInt = (id, fallback = 0) => {
    const raw = String(qs(`#${id}`)?.value ?? '').trim();
    const n = Number(raw);
    return Number.isFinite(n) ? n : fallback;
  };
  const readText = (id, fallback = '') => {
    const val = qs(`#${id}`)?.value;
    if (val === undefined || val === null) return fallback;
    return String(val).trim();
  };
  const splitInputList = (raw) => String(raw || '')
    .split(/[\n,，]+/)
    .map((s) => s.trim())
    .filter(Boolean);
  const parseJsonText = (raw, fallback = {}) => {
    const text = String(raw || '').trim();
    if (!text) return fallback;
    return JSON.parse(text);
  };
  const toUnixTs = (raw, fallback = 0) => {
    const text = String(raw || '').trim();
    if (!text) return fallback;
    const ms = Date.parse(text);
    if (!Number.isFinite(ms)) return fallback;
    return Math.floor(ms / 1000);
  };
  const withQuery = (url, params = {}) => {
    const q = new URLSearchParams();
    Object.entries(params).forEach(([k, v]) => {
      if (v === undefined || v === null || v === '') return;
      q.set(k, String(v));
    });
    const query = q.toString();
    if (!query) return url;
    return url.includes('?') ? `${url}&${query}` : `${url}?${query}`;
  };
  const v2Request = async (endpoint, { method = 'GET', body, query } = {}) => {
    if (!me?.is_super_admin) throw new Error('需要超级管理员权限');
    if (!securePath) throw new Error('未找到后台路径，无法执行该操作');
    const url = withQuery(buildV2(endpoint), query || {});
    return apiFetch(url, { method, body });
  };

  const v2SystemOut = qs('#v2_system_out');
  const v2TrafficOut = qs('#v2_traffic_out');
  const v2TicketOut = qs('#v2_ticket_out');
  const v2ReloadBtn = qs('#admV2ReloadBtn');
  if (v2ReloadBtn) {
    v2ReloadBtn.addEventListener('click', async () => {
      await rerender('已刷新迁移模块');
    });
  }

  const v2PlanCreateBtn = qs('#v2PlanCreateBtn');
  if (v2PlanCreateBtn) {
    v2PlanCreateBtn.addEventListener('click', async () => {
      try {
        const name = readText('v2_plan_name');
        if (!name) throw new Error('套餐名称不能为空');
        const transferGb = Math.max(1, readInt('v2_plan_transfer_gb', 100));
        const priceFieldMap = {
          month_price: 'v2_plan_price_month',
          quarter_price: 'v2_plan_price_quarter',
          half_year_price: 'v2_plan_price_half_year',
          year_price: 'v2_plan_price_year',
          two_year_price: 'v2_plan_price_two_year',
          three_year_price: 'v2_plan_price_three_year',
          onetime_price: 'v2_plan_price_onetime',
          reset_price: 'v2_plan_price_reset'
        };
        const prices = {};
        Object.entries(priceFieldMap).forEach(([periodKey, inputId]) => {
          const n = Number(readText(inputId, '0'));
          if (Number.isFinite(n) && n > 0) {
            prices[periodKey] = Number(n.toFixed(2));
          }
        });
        if (!Object.keys(prices).length) {
          throw new Error('至少设置一个周期价格');
        }
        const tags = readText('v2_plan_tags', '')
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean);
        const payload = {
          name,
          content: readText('v2_plan_content'),
          transfer_enable: transferGb,
          group_id: readInt('v2_plan_group_id', 0) || 0,
          speed_limit: Math.max(0, readInt('v2_plan_speed', 0)),
          device_limit: Math.max(0, readInt('v2_plan_device', 0)),
          capacity_limit: Math.max(0, readInt('v2_plan_capacity', 0)),
          prices,
          reset_traffic_method: readInt('v2_plan_reset_method', 0),
          tags,
          show: readText('v2_plan_show', '1') === '1',
          sell: readText('v2_plan_sell', '1') === '1',
          renew: readText('v2_plan_renew', '1') === '1',
          sort: readInt('v2_plan_sort', 0),
        };
        await v2Request('plan/save', { method: 'POST', body: payload });
        await rerender('套餐创建成功');
      } catch (e) {
        await rerender(e.message || '创建套餐失败');
      }
    });
  }

  qsa('button[data-v2-plan-toggle]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-plan-toggle'));
    const key = String(btn.getAttribute('data-toggle-key') || '');
    if (!id || !['show', 'sell', 'renew'].includes(key)) return;
    const current = migratedV2Plans.find((p) => Number(p.id) === id);
    if (!current) return;
    try {
      await v2Request('plan/update', {
        method: 'POST',
        body: { id, [key]: current[key] ? 0 : 1 }
      });
      await rerender(`已更新套餐 ${id} 的 ${key}`);
    } catch (e) {
      await rerender(e.message || '更新套餐失败');
    }
  }));
  qsa('button[data-v2-plan-drop]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-plan-drop'));
    if (!id) return;
    if (!confirm(`确认删除套餐 #${id}？`)) return;
    try {
      await v2Request('plan/drop', { method: 'POST', body: { id } });
      await rerender(`已删除套餐 #${id}`);
    } catch (e) {
      await rerender(e.message || '删除套餐失败');
    }
  }));

  const v2PaymentCreateBtn = qs('#v2PaymentCreateBtn');
  if (v2PaymentCreateBtn) {
    v2PaymentCreateBtn.addEventListener('click', async () => {
      try {
        const name = readText('v2_payment_name');
        const payment = readText('v2_payment_method');
        if (!name || !payment) throw new Error('请完整填写支付方式名称和类型');
        const payload = {
          name,
          payment,
          icon: readText('v2_payment_icon'),
          notify_domain: readText('v2_payment_notify_domain') || null,
          config: parseJsonText(readText('v2_payment_config', '{}'), {})
        };
        await v2Request('payment/save', { method: 'POST', body: payload });
        await rerender('支付方式新增成功');
      } catch (e) {
        await rerender(e.message || '新增支付方式失败');
      }
    });
  }
  qsa('button[data-v2-payment-toggle]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-payment-toggle'));
    if (!id) return;
    try {
      await v2Request('payment/show', { method: 'POST', body: { id } });
      await rerender(`支付方式 #${id} 状态已更新`);
    } catch (e) {
      await rerender(e.message || '更新支付方式失败');
    }
  }));
  qsa('button[data-v2-payment-drop]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-payment-drop'));
    if (!id) return;
    if (!confirm(`确认删除支付方式 #${id}？`)) return;
    try {
      await v2Request('payment/drop', { method: 'POST', body: { id } });
      await rerender(`支付方式 #${id} 已删除`);
    } catch (e) {
      await rerender(e.message || '删除支付方式失败');
    }
  }));

  const v2NoticeCreateBtn = qs('#v2NoticeCreateBtn');
  if (v2NoticeCreateBtn) {
    v2NoticeCreateBtn.addEventListener('click', async () => {
      try {
        const title = readText('v2_notice_title');
        const content = readText('v2_notice_content');
        if (!title || !content) throw new Error('公告标题和内容不能为空');
        const payload = {
          title,
          content,
          tags: splitInputList(readText('v2_notice_tags')),
          show: 1,
          popup: 0
        };
        await v2Request('notice/save', { method: 'POST', body: payload });
        await rerender('公告发布成功');
      } catch (e) {
        await rerender(e.message || '发布公告失败');
      }
    });
  }
  qsa('button[data-v2-notice-toggle]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-notice-toggle'));
    if (!id) return;
    try {
      await v2Request('notice/show', { method: 'POST', body: { id } });
      await rerender(`公告 #${id} 显示状态已更新`);
    } catch (e) {
      await rerender(e.message || '更新公告失败');
    }
  }));
  qsa('button[data-v2-notice-drop]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-notice-drop'));
    if (!id) return;
    if (!confirm(`确认删除公告 #${id}？`)) return;
    try {
      await v2Request('notice/drop', { method: 'POST', body: { id } });
      await rerender(`公告 #${id} 已删除`);
    } catch (e) {
      await rerender(e.message || '删除公告失败');
    }
  }));

  qsa('button[data-v2-ticket-detail]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-ticket-detail'));
    if (!id) return;
    try {
      const detail = await v2Request('ticket/fetch', { query: { id } });
      if (v2TicketOut) v2TicketOut.textContent = safeJson(detail);
      const replyId = qs('#v2_ticket_reply_id');
      if (replyId) replyId.value = String(id);
    } catch (e) {
      if (v2TicketOut) v2TicketOut.textContent = e.message || '读取工单失败';
    }
  }));
  qsa('button[data-v2-ticket-close]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-ticket-close'));
    if (!id) return;
    if (!confirm(`确认关闭工单 #${id}？`)) return;
    try {
      await v2Request('ticket/close', { method: 'POST', body: { id } });
      await rerender(`工单 #${id} 已关闭`);
    } catch (e) {
      await rerender(e.message || '关闭工单失败');
    }
  }));
  const v2TicketReplyBtn = qs('#v2TicketReplyBtn');
  if (v2TicketReplyBtn) {
    v2TicketReplyBtn.addEventListener('click', async () => {
      try {
        const id = readInt('v2_ticket_reply_id', 0);
        const message = readText('v2_ticket_reply_message');
        if (!id) throw new Error('请输入工单 ID');
        if (!message) throw new Error('回复内容不能为空');
        await v2Request('ticket/reply', { method: 'POST', body: { id, message } });
        await rerender(`工单 #${id} 回复成功`);
      } catch (e) {
        if (v2TicketOut) v2TicketOut.textContent = e.message || '回复失败';
      }
    });
  }

  const v2CouponCreateBtn = qs('#v2CouponCreateBtn');
  if (v2CouponCreateBtn) {
    v2CouponCreateBtn.addEventListener('click', async () => {
      try {
        const name = readText('v2_coupon_name');
        const type = readInt('v2_coupon_type', 1);
        const value = Math.max(1, readInt('v2_coupon_value', 100));
        const startedAt = toUnixTs(readText('v2_coupon_start'), Math.floor(Date.now() / 1000));
        const endedAt = toUnixTs(readText('v2_coupon_end'), startedAt + 86400 * 30);
        if (!name) throw new Error('优惠券名称不能为空');
        if (endedAt <= startedAt) throw new Error('结束时间必须晚于开始时间');
        const payload = {
          name,
          type,
          value,
          started_at: startedAt,
          ended_at: endedAt,
          limit_use: Math.max(0, readInt('v2_coupon_limit_use', 0)),
          limit_use_with_user: Math.max(0, readInt('v2_coupon_limit_user', 0)),
          limit_plan_ids: splitInputList(readText('v2_coupon_plan_ids')).map((v) => Number(v)).filter((v) => Number.isFinite(v) && v > 0),
          limit_period: []
        };
        await v2Request('coupon/generate', { method: 'POST', body: payload });
        await rerender('优惠券创建成功');
      } catch (e) {
        await rerender(e.message || '创建优惠券失败');
      }
    });
  }
  qsa('button[data-v2-coupon-toggle]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-coupon-toggle'));
    if (!id) return;
    try {
      await v2Request('coupon/show', { method: 'POST', body: { id } });
      await rerender(`优惠券 #${id} 显示状态已更新`);
    } catch (e) {
      await rerender(e.message || '更新优惠券失败');
    }
  }));
  qsa('button[data-v2-coupon-drop]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-coupon-drop'));
    if (!id) return;
    if (!confirm(`确认删除优惠券 #${id}？`)) return;
    try {
      await v2Request('coupon/drop', { method: 'POST', body: { id } });
      await rerender(`优惠券 #${id} 已删除`);
    } catch (e) {
      await rerender(e.message || '删除优惠券失败');
    }
  }));

  const v2GiftGenerateBtn = qs('#v2GiftGenerateBtn');
  if (v2GiftGenerateBtn) {
    v2GiftGenerateBtn.addEventListener('click', async () => {
      try {
        const templateId = readInt('v2_gift_template_id', 0);
        const count = Math.max(1, readInt('v2_gift_count', 10));
        const prefix = readText('v2_gift_prefix', 'GC') || 'GC';
        const maxUsage = Math.max(1, readInt('v2_gift_max_usage', 1));
        if (!templateId) throw new Error('请选择礼品卡模板');
        await v2Request('gift-card/generate-codes', {
          method: 'POST',
          body: {
            template_id: templateId,
            count,
            prefix,
            max_usage: maxUsage
          }
        });
        await rerender('礼品卡兑换码已生成');
      } catch (e) {
        await rerender(e.message || '生成兑换码失败');
      }
    });
  }
  qsa('button[data-v2-gift-toggle]').forEach((btn) => btn.addEventListener('click', async () => {
    const id = Number(btn.getAttribute('data-v2-gift-toggle'));
    const action = String(btn.getAttribute('data-v2-gift-action') || 'disable');
    if (!id) return;
    try {
      await v2Request('gift-card/toggle-code', {
        method: 'POST',
        body: { id, action }
      });
      await rerender(`兑换码 #${id} 状态已更新`);
    } catch (e) {
      await rerender(e.message || '更新兑换码失败');
    }
  }));

  qsa('button[data-v2-plugin-act]').forEach((btn) => btn.addEventListener('click', async () => {
    const act = String(btn.getAttribute('data-v2-plugin-act') || '').trim();
    const code = String(btn.getAttribute('data-v2-plugin-code') || '').trim();
    if (!act || !code) return;
    const allowed = ['install', 'uninstall', 'enable', 'disable', 'upgrade'];
    if (!allowed.includes(act)) return;
    if ((act === 'uninstall' || act === 'disable') && !confirm(`确认执行 ${act}：${code}？`)) return;
    try {
      await v2Request(`plugin/${act}`, { method: 'POST', body: { code } });
      await rerender(`插件 ${code} 已执行 ${act}`);
    } catch (e) {
      await rerender(e.message || `插件 ${act} 失败`);
    }
  }));

  const v2SystemLogsBtn = qs('#v2SystemLogsBtn');
  if (v2SystemLogsBtn) {
    v2SystemLogsBtn.addEventListener('click', async () => {
      try {
        const logs = await v2Request('system/getSystemLog', {
          query: { current: 1, page_size: 20 }
        });
        if (v2SystemOut) v2SystemOut.textContent = safeJson(logs);
      } catch (e) {
        if (v2SystemOut) v2SystemOut.textContent = e.message || '读取系统日志失败';
      }
    });
  }
  const v2SystemClearBtn = qs('#v2SystemClearBtn');
  if (v2SystemClearBtn) {
    v2SystemClearBtn.addEventListener('click', async () => {
      try {
        if (!confirm('确认执行日志清理？')) return;
        const days = Math.max(0, readInt('v2_log_days', 30));
        const level = readText('v2_log_level', 'all') || 'all';
        const limit = Math.max(100, Math.min(10000, readInt('v2_log_limit', 1000)));
        const resp = await v2Request('system/clearSystemLog', {
          method: 'POST',
          body: { days, level, limit }
        });
        if (v2SystemOut) v2SystemOut.textContent = safeJson(resp);
        await rerender('日志清理已执行');
      } catch (e) {
        if (v2SystemOut) v2SystemOut.textContent = e.message || '日志清理失败';
      }
    });
  }

  const v2TrafficHistoryBtn = qs('#v2TrafficHistoryBtn');
  if (v2TrafficHistoryBtn) {
    v2TrafficHistoryBtn.addEventListener('click', async () => {
      try {
        const uid = readInt('v2_traffic_user_id', 0);
        if (!uid) throw new Error('请输入用户 ID');
        const resp = await v2Request(`traffic-reset/user/${uid}/history`, {
          query: { limit: 20 }
        });
        if (v2TrafficOut) v2TrafficOut.textContent = safeJson(resp);
      } catch (e) {
        if (v2TrafficOut) v2TrafficOut.textContent = e.message || '读取用户历史失败';
      }
    });
  }
  const v2TrafficResetBtn = qs('#v2TrafficResetBtn');
  if (v2TrafficResetBtn) {
    v2TrafficResetBtn.addEventListener('click', async () => {
      try {
        const userId = readInt('v2_traffic_user_id', 0);
        if (!userId) throw new Error('请输入用户 ID');
        const reason = readText('v2_traffic_reason');
        const resp = await v2Request('traffic-reset/reset-user', {
          method: 'POST',
          body: { user_id: userId, reason }
        });
        if (v2TrafficOut) v2TrafficOut.textContent = safeJson(resp);
        await rerender(`用户 #${userId} 流量重置已执行`);
      } catch (e) {
        if (v2TrafficOut) v2TrafficOut.textContent = e.message || '重置流量失败';
      }
    });
  }

  const userLimitOut = qs('#adm_user_limit_detail');
  const loadUserIpLimits = async () => {
    const uid = Number(qs('#adm_uid').value);
    if (!uid) throw new Error('请先输入用户 ID');

    const [detailRes, concurrentRes] = await Promise.all([
      apiFetch(`/api/v1/admin/users/${uid}/limits`),
      apiFetch(`/api/v1/admin/users/${uid}/concurrent-ip-limit`)
    ]);

    const detail = unwrap(detailRes) || {};
    const current = detail?.individual_limits || {};
    const effective = detail?.effective_limits || {};
    const concurrent = unwrap(concurrentRes) || {};

    qs('#adm_user_device_limit').value = String(current?.device_limit ?? effective?.device_limit ?? 0);
    qs('#adm_limit').value = String(concurrent?.concurrent_ip_limit ?? 0);

    userLimitOut.textContent = safeJson({
      user_id: uid,
      same_user_diff_ip_limit: current?.device_limit ?? effective?.device_limit ?? 0,
      same_ip_cross_nodes_limit: concurrent?.concurrent_ip_limit ?? 0,
      raw: { detail, concurrent }
    });

    return { detail, concurrent };
  };

  qs('#adm_load_ip_btn').addEventListener('click', async () => {
    try {
      await loadUserIpLimits();
    } catch (e) {
      userLimitOut.textContent = e.message || '读取失败';
    }
  });

  qs('#adm_set_btn').addEventListener('click', async () => {
    try {
      const uid = Number(qs('#adm_uid').value);
      if (!uid) throw new Error('请先输入用户 ID');

      // 先读取，保证不会意外覆盖其它个人限制字段
      const currentRes = await apiFetch(`/api/v1/admin/users/${uid}/limits`);
      const currentDetail = unwrap(currentRes) || {};
      const current = currentDetail?.individual_limits || {};

      const sameUserDiffIp = readInt('adm_user_device_limit', 0);
      const sameIpCrossNodes = readInt('adm_limit', 0);

      await apiFetch(`/api/v1/admin/users/${uid}/limits`, {
        method: 'POST',
        body: {
          speed_limit_up: Number(current?.speed_limit_up ?? 0),
          speed_limit_down: Number(current?.speed_limit_down ?? 0),
          device_limit: Math.max(0, sameUserDiffIp),
          connection_limit: Number(current?.connection_limit ?? 0),
        }
      });

      await apiFetch(`/api/v1/admin/users/${uid}/concurrent-ip-limit`, {
        method: 'PUT',
        body: { concurrent_ip_limit: Math.max(0, sameIpCrossNodes) }
      });

      await loadUserIpLimits();
      await rerender('同用户异 IP 限制已保存');
    } catch (e) {
      await rerender(e.message || '保存失败');
    }
  });

  const getRecommendedOauthCallback = () => {
    const fromSite = String(siteCfg.app_url || '').trim();
    const fromMeta = String(window.__APP__?.baseUrl || '').trim();
    const base = (fromSite || fromMeta || location.origin).replace(/\/+$/, '');
    return `${base}/api/v1/passport/oauth2/linux-do/callback`;
  };

  const reloadConfigBtn = qs('#admReloadConfigBtn');
  if (reloadConfigBtn) {
    reloadConfigBtn.addEventListener('click', async () => {
      await rerender('已重新读取安全配置');
    });
  }

  const oauthCallbackBtn = qs('#admOauthCallbackBtn');
  if (oauthCallbackBtn) {
    oauthCallbackBtn.addEventListener('click', () => {
      const redirectInput = qs('#oauth_linux_do_redirect_uri');
      if (redirectInput) redirectInput.value = getRecommendedOauthCallback();
    });
  }

  const captchaEnableSelect = qs('#sec_captcha_enable');
  const captchaTypeSelect = qs('#sec_captcha_type');
  const captchaHint = qs('#sec_captcha_provider_hint');
  const syncCaptchaProviderFields = () => {
    const enabled = String(captchaEnableSelect?.value || '0') === '1';
    const provider = String(captchaTypeSelect?.value || 'recaptcha');
    qsa('.captcha-provider-field').forEach((field) => {
      const allowed = String(field.getAttribute('data-captcha-provider') || '')
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean);
      const visible = enabled && allowed.includes(provider);
      field.classList.toggle('hidden', !visible);
    });
    if (captchaHint) {
      if (!enabled) {
        captchaHint.textContent = '人机验证已关闭，下面字段暂不生效。';
      } else if (provider === 'recaptcha') {
        captchaHint.textContent = '当前为 Google 验证（普通），仅需填写普通密钥。';
      } else if (provider === 'recaptcha-v3') {
        captchaHint.textContent = '当前为 Google 验证（无感），请填写 v3 密钥和阈值。';
      } else if (provider === 'turnstile') {
        captchaHint.textContent = '当前为 Cloudflare Turnstile，请填写站点密钥和密钥。';
      } else {
        captchaHint.textContent = '请先选择有效的人机验证服务商。';
      }
    }
  };
  if (captchaEnableSelect) captchaEnableSelect.addEventListener('change', syncCaptchaProviderFields);
  if (captchaTypeSelect) captchaTypeSelect.addEventListener('change', syncCaptchaProviderFields);
  syncCaptchaProviderFields();

  const saveSecurityBtn = qs('#admSaveSecurityBtn');
  if (saveSecurityBtn) {
    saveSecurityBtn.addEventListener('click', async () => {
      try {
        if (!securePath) throw new Error('未找到后台路径，无法保存');
        const payload = {
          safe_mode_enable: readBoolSelect('sec_safe_mode_enable'),
          email_verify: readBoolSelect('sec_email_verify'),
          register_mode: readText('sec_register_mode', 'all') || 'all',
          secure_path: readText('sec_secure_path'),
          force_oauth2_login: readBoolSelect('sec_force_oauth2_login'),
          login_token_expire_days: Math.max(0, Math.min(3650, readInt('sec_login_token_expire_days', 365))),
          email_whitelist_enable: readBoolSelect('sec_email_whitelist_enable'),
          email_whitelist_suffix: splitInputList(readText('sec_email_whitelist_suffix')),
          email_gmail_limit_enable: readBoolSelect('sec_email_gmail_limit_enable'),
          register_limit_by_ip_enable: readBoolSelect('sec_register_limit_by_ip_enable'),
          register_limit_count: Math.max(1, readInt('sec_register_limit_count', 3)),
          register_limit_expire: Math.max(1, readInt('sec_register_limit_expire', 60)),
          password_limit_enable: readBoolSelect('sec_password_limit_enable'),
          password_limit_count: Math.max(1, readInt('sec_password_limit_count', 5)),
          password_limit_expire: Math.max(1, readInt('sec_password_limit_expire', 60)),
          captcha_enable: readBoolSelect('sec_captcha_enable'),
          captcha_type: readText('sec_captcha_type', 'recaptcha'),
          recaptcha_key: readText('sec_recaptcha_key'),
          recaptcha_site_key: readText('sec_recaptcha_site_key'),
          recaptcha_v3_secret_key: readText('sec_recaptcha_v3_secret_key'),
          recaptcha_v3_site_key: readText('sec_recaptcha_v3_site_key'),
          recaptcha_v3_score_threshold: Math.max(0, Math.min(1, Number(readText('sec_recaptcha_v3_score_threshold', '0.5')) || 0.5)),
          turnstile_secret_key: readText('sec_turnstile_secret_key'),
          turnstile_site_key: readText('sec_turnstile_site_key'),
          pow_enable: readBoolSelect('sec_pow_enable'),
          pow_auto_scale_enable: readBoolSelect('sec_pow_auto_scale_enable'),
          pow_difficulty: Math.max(1, Math.min(8, readInt('sec_pow_difficulty', 4))),
          pow_auto_max_difficulty: Math.max(1, Math.min(8, readInt('sec_pow_auto_max_difficulty', 7))),
          pow_ttl: Math.max(30, Math.min(600, readInt('sec_pow_ttl', 120))),
          pow_seed_salt: readText('sec_pow_seed_salt'),
          pow_base_value: readText('sec_pow_base_value', 'portal') || 'portal',
          pow_require_ja3: readBoolSelect('sec_pow_require_ja3'),
        };
        await apiFetch(buildV2('config/save'), { method: 'POST', body: payload });
        await rerender('安全设置已保存');
      } catch (e) {
        await rerender(e.message || '保存安全设置失败');
      }
    });
  }

  const saveOauthBtn = qs('#admSaveOauthBtn');
  if (saveOauthBtn) {
    saveOauthBtn.addEventListener('click', async () => {
      try {
        if (!securePath) throw new Error('未找到后台路径，无法保存');
        const payload = {
          oauth_linux_do_enable: readBoolSelect('oauth_linux_do_enable'),
          oauth_linux_do_client_id: readText('oauth_linux_do_client_id'),
          oauth_linux_do_client_secret: readText('oauth_linux_do_client_secret'),
          oauth_linux_do_redirect_uri: readText('oauth_linux_do_redirect_uri') || getRecommendedOauthCallback(),
        };
        await apiFetch(buildV2('config/save'), { method: 'POST', body: payload });
        await rerender('OAuth2 设置已保存');
      } catch (e) {
        await rerender(e.message || '保存 OAuth2 设置失败');
      }
    });
  }

  const saveSiteRoutingBtn = qs('#admSaveSiteRoutingBtn');
  if (saveSiteRoutingBtn) {
    saveSiteRoutingBtn.addEventListener('click', async () => {
      try {
        if (!securePath) throw new Error('未找到后台路径，无法保存');
        const payload = {
          subscribe_url: readText('site_subscribe_url'),
          subscribe_root_domains: readText('site_subscribe_root_domains'),
        };
        await apiFetch(buildV2('config/save'), { method: 'POST', body: payload });
        await rerender('注册与订阅入口已保存');
      } catch (e) {
        await rerender(e.message || '保存注册与订阅入口失败');
      }
    });
  }

  const saveAppClientBtn = qs('#admSaveAppClientBtn');
  if (saveAppClientBtn) {
    saveAppClientBtn.addEventListener('click', async () => {
      try {
        if (!securePath) throw new Error('未找到后台路径，无法保存');
        const payload = {
          windows_version: readText('app_windows_version'),
          windows_download_url: readText('app_windows_download_url'),
          macos_version: readText('app_macos_version'),
          macos_download_url: readText('app_macos_download_url'),
          android_version: readText('app_android_version'),
          android_download_url: readText('app_android_download_url'),
        };
        await apiFetch(buildV2('config/save'), { method: 'POST', body: payload });
        await rerender('客户端版本与下载地址已保存');
      } catch (e) {
        await rerender(e.message || '保存客户端版本设置失败');
      }
    });
  }

  const saveSystemRetentionBtn = qs('#admSaveSystemRetentionBtn');
  if (saveSystemRetentionBtn) {
    saveSystemRetentionBtn.addEventListener('click', async () => {
      try {
        if (!securePath) throw new Error('未找到后台路径，无法保存');
        const payload = {
          rotate_subscription_credentials_daily: readBoolSelect('sys_rotate_subscription_credentials_daily'),
          refund_dispute_enable: readBoolSelect('sys_refund_dispute_enable'),
          node_traffic_records_retention_days: Math.max(1, readInt('sys_node_traffic_records_retention_days', 7)),
          user_traffic_usage_logs_retention_days: Math.max(1, readInt('sys_user_traffic_usage_logs_retention_days', 7)),
          tcping_samples_retention_days: Math.max(1, readInt('sys_tcping_samples_retention_days', 7)),
          tcping_alerts_retention_days: Math.max(1, readInt('sys_tcping_alerts_retention_days', 7)),
          audit_logs_retention_days: Math.max(1, readInt('sys_audit_logs_retention_days', 7)),
        };
        await apiFetch(buildV2('config/save'), { method: 'POST', body: payload });
        await rerender('系统保留策略已保存');
      } catch (e) {
        await rerender(e.message || '保存系统保留策略失败');
      }
    });
  }

  const fmtSingboxBtn = qs('#admFmtSingboxTplBtn');
  if (fmtSingboxBtn) {
    fmtSingboxBtn.addEventListener('click', () => {
      try {
        const area = qs('#proxy_tpl_singbox');
        if (!area) return;
        const parsed = JSON.parse(String(area.value || '{}'));
        area.value = JSON.stringify(parsed, null, 2);
        alert('SingBox JSON 已格式化');
      } catch (e) {
        alert('SingBox 模板不是合法 JSON，请检查后再保存');
      }
    });
  }

  const saveProxyTplBtn = qs('#admSaveProxyTplBtn');
  if (saveProxyTplBtn) {
    saveProxyTplBtn.addEventListener('click', async () => {
      try {
        if (!securePath) throw new Error('未找到后台路径，无法保存');

        const singboxRaw = String(qs('#proxy_tpl_singbox')?.value ?? '');
        // 提前校验，避免把无效 JSON 写进配置后导致订阅下发异常。
        JSON.parse(singboxRaw || '{}');

        const payload = {
          subscribe_template_singbox: singboxRaw,
          subscribe_template_clash: String(qs('#proxy_tpl_clash')?.value ?? ''),
          subscribe_template_clashmeta: String(qs('#proxy_tpl_clashmeta')?.value ?? ''),
          subscribe_template_stash: String(qs('#proxy_tpl_stash')?.value ?? ''),
          subscribe_template_surge: String(qs('#proxy_tpl_surge')?.value ?? ''),
          subscribe_template_surfboard: String(qs('#proxy_tpl_surfboard')?.value ?? ''),
        };

        await apiFetch(buildV2('config/save'), { method: 'POST', body: payload });
        await rerender('代理模板已保存');
      } catch (e) {
        await rerender(e.message || '保存代理模板失败');
      }
    });
  }

  const indOut = qs('#adm_individual_detail');
  const loadIndividual = async () => {
    const uid = Number(qs('#adm_ind_uid').value);
    if (!uid) throw new Error('请输入有效 User ID');
    const res = await apiFetch(`/api/v1/admin/users/${uid}/limits`);
    const detail = unwrap(res) || {};
    const current = detail?.individual_limits || {};
    qs('#adm_ind_up').value = current?.speed_limit_up ?? '';
    qs('#adm_ind_down').value = current?.speed_limit_down ?? '';
    qs('#adm_ind_dev').value = current?.device_limit ?? '';
    qs('#adm_ind_conn').value = current?.connection_limit ?? '';
    indOut.textContent = safeJson(detail);
  };
  qs('#admIndLoadBtn').addEventListener('click', async () => {
    try {
      await loadIndividual();
    } catch (e) {
      indOut.textContent = e.message || '读取失败';
    }
  });
  qs('#admIndSaveBtn').addEventListener('click', async () => {
    try {
      const uid = Number(qs('#adm_ind_uid').value);
      if (!uid) throw new Error('请输入有效 User ID');
      const payload = {
        speed_limit_up: toNumOrNull(qs('#adm_ind_up').value),
        speed_limit_down: toNumOrNull(qs('#adm_ind_down').value),
        device_limit: toNumOrNull(qs('#adm_ind_dev').value),
        connection_limit: toNumOrNull(qs('#adm_ind_conn').value),
      };
      const res = await apiFetch(`/api/v1/admin/users/${uid}/limits`, { method: 'POST', body: payload });
      indOut.textContent = safeJson(unwrap(res));
      alert('用户个人限制已保存');
    } catch (e) {
      indOut.textContent = e.message || '保存失败';
    }
  });
  qs('#admIndDeleteBtn').addEventListener('click', async () => {
    try {
      const uid = Number(qs('#adm_ind_uid').value);
      if (!uid) throw new Error('请输入有效 User ID');
      if (!confirm('确认删除该用户个人限制并恢复组限制？')) return;
      const res = await apiFetch(`/api/v1/admin/users/${uid}/limits`, { method: 'DELETE' });
      indOut.textContent = safeJson(unwrap(res));
      alert('已删除个人限制');
    } catch (e) {
      indOut.textContent = e.message || '删除失败';
    }
  });

  const groupRefreshBtn = qs('#admGroupRefresh');
  if (groupRefreshBtn) groupRefreshBtn.addEventListener('click', () => rerender('分组限制已刷新'));
  const groupDefaultsBtn = qs('#admGroupDefaults');
  if (groupDefaultsBtn) groupDefaultsBtn.addEventListener('click', async () => {
    try {
      await apiFetch('/api/v1/admin/group-limits/defaults/apply', { method: 'POST', body: {} });
      await rerender('默认模板已应用');
    } catch (e) {
      await rerender(e.message || '应用失败');
    }
  });

  qsa('button[data-gl-save]').forEach((btn) => btn.addEventListener('click', async () => {
    const trust = Number(btn.getAttribute('data-gl-save'));
    try {
      await apiFetch('/api/v1/admin/group-limits', {
        method: 'POST',
        body: {
          trust_level: trust,
          speed_limit_up: toNumOrNull(qs(`#gl_up_${trust}`).value),
          speed_limit_down: toNumOrNull(qs(`#gl_down_${trust}`).value),
          device_limit: toNumOrNull(qs(`#gl_dev_${trust}`).value),
          connection_limit: toNumOrNull(qs(`#gl_conn_${trust}`).value),
        }
      });
      await rerender(`已保存 trust_level=${trust}`);
    } catch (e) {
      await rerender(e.message || `保存失败 trust_level=${trust}`);
    }
  }));
  qsa('button[data-gl-del]').forEach((btn) => btn.addEventListener('click', async () => {
    const trust = Number(btn.getAttribute('data-gl-del'));
    if (!confirm(`确认删除 trust_level=${trust} 配置？`)) return;
    try {
      await apiFetch(`/api/v1/admin/group-limits/${trust}`, { method: 'DELETE' });
      await rerender(`已删除 trust_level=${trust}`);
    } catch (e) {
      await rerender(e.message || `删除失败 trust_level=${trust}`);
    }
  }));

  const apiDetail = qs('#adm_api_detail');
  const apiSearchBody = qs('#adm_api_search_tbody');
  const apiSearchState = { items: [] };
  const renderApiSearchRows = () => {
    if (!apiSearchState.items.length) {
      apiSearchBody.innerHTML = '<tr><td colspan="5" class="muted">未检索到结果。</td></tr>';
      return;
    }
    apiSearchBody.innerHTML = apiSearchState.items.map((u) => html`
      <tr>
        <td>${u.id}</td>
        <td class="muted">${escapeHtml(u.email || '-')}</td>
        <td class="muted">${escapeHtml(u.linux_do_username || '-')}</td>
        <td class="muted">${escapeHtml(u.api_key_prefix || '-')}</td>
        <td class="row end">
          <button class="btn small" data-ak-act="detail" data-ak-id="${u.id}">详情</button>
          <button class="btn small primary" data-ak-act="generate" data-ak-id="${u.id}">生成</button>
          <button class="btn small danger" data-ak-act="reset" data-ak-id="${u.id}">重置</button>
        </td>
      </tr>
    `).join('');
  };
  const searchApiUsers = async () => {
    const query = qs('#adm_api_query').value.trim();
    if (query.length < 3) throw new Error('关键词至少 3 位');
    const perPage = Number(qs('#adm_api_per_page').value) || 20;
    const res = await apiFetch(`/api/v1/admin/api-keys/search?query=${encodeURIComponent(query)}&per_page=${perPage}`);
    const payload = unwrap(res) || {};
    apiSearchState.items = Array.isArray(payload?.data) ? payload.data : [];
    renderApiSearchRows();
    apiDetail.textContent = safeJson(payload);
  };
  qs('#admApiSearchBtn').addEventListener('click', async () => {
    try {
      await searchApiUsers();
    } catch (e) {
      apiDetail.textContent = e.message || '搜索失败';
    }
  });
  apiSearchBody.addEventListener('click', async (e) => {
    const btn = e.target.closest('button[data-ak-act]');
    if (!btn) return;
    const id = Number(btn.getAttribute('data-ak-id'));
    const act = btn.getAttribute('data-ak-act');
    if (!id) return;
    try {
      if (act === 'detail') {
        const res = await apiFetch(`/api/v1/admin/users/${id}/api-key`);
        apiDetail.textContent = safeJson(unwrap(res));
      } else if (act === 'generate') {
        const res = await apiFetch(`/api/v1/admin/users/${id}/api-key/generate`, { method: 'POST', body: {} });
        apiDetail.textContent = safeJson(unwrap(res));
      } else if (act === 'reset') {
        if (!confirm(`确认重置用户 ${id} 的 API Key？`)) return;
        const res = await apiFetch(`/api/v1/admin/users/${id}/api-key/reset`, { method: 'POST', body: {} });
        apiDetail.textContent = safeJson(unwrap(res));
      }
      const q = qs('#adm_api_query').value.trim();
      if (q.length >= 3) await searchApiUsers();
    } catch (err) {
      apiDetail.textContent = err.message || '操作失败';
    }
  });

  qs('#admApiRefreshStats').addEventListener('click', () => rerender('API Key 统计已刷新'));
  qs('#admApiBatchGenerate').addEventListener('click', async () => {
    try {
      const res = await apiFetch('/api/v1/admin/api-keys/batch-generate', { method: 'POST', body: {} });
      apiDetail.textContent = safeJson(unwrap(res));
      await rerender('批量生成已完成');
    } catch (e) {
      apiDetail.textContent = e.message || '批量生成失败';
    }
  });
  qs('#admApiCleanup').addEventListener('click', async () => {
    try {
      const res = await apiFetch('/api/v1/admin/api-keys/cleanup', { method: 'POST', body: {} });
      apiDetail.textContent = safeJson(unwrap(res));
      await rerender('清理已完成');
    } catch (e) {
      apiDetail.textContent = e.message || '清理失败';
    }
  });

  const admSponsorSaveBtn = qs('#admSponsorSaveBtn');
  if (admSponsorSaveBtn) {
    admSponsorSaveBtn.addEventListener('click', async () => {
      try {
        const key = qs('#sp_key').value.trim();
        if (!key) throw new Error('key 不能为空（出于安全原因不做回显，请手动输入）');
        await apiFetch('/api/v1/admin/sponsor-epay', {
          method: 'PUT',
          body: {
            url: qs('#sp_url').value.trim(),
            submit_path: qs('#sp_submit').value.trim() || '/pay/submit.php',
            pid: qs('#sp_pid').value.trim(),
            key,
            use_post: qs('#sp_post').value === '1',
            sitename: qs('#sp_site').value.trim(),
            device: qs('#sp_device').value.trim(),
          }
        });
        await rerender('赞助收款配置已保存');
      } catch (e) {
        await rerender(e.message || '保存赞助收款失败');
      }
    });
  }

  const openForm = (mode, id) => {
    const plan = id ? nodePlans.find(p => String(p.id) === String(id)) : null;
    showAdminNodePlanFormModal({
      mode,
      plan,
      nodeOptions: adminNodeOptions,
      onSubmit: async (payload) => {
        if (mode === 'new') {
          await apiFetch('/api/v1/admin/node-plans', { method: 'POST', body: payload });
        } else {
          await apiFetch('/api/v1/admin/node-plans/' + id, { method: 'PUT', body: payload });
        }
        await renderAdmin();
      }
    });
  };

  const newBtn = qs('#admNewNodePlan');
  if (newBtn) newBtn.addEventListener('click', () => openForm('new'));
  qsa('button[data-adm-copy-plan-link]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      try {
        await copyText(btn.getAttribute('data-adm-copy-plan-link') || '');
        alert('专属购买链接已复制');
      } catch (e) {
        alert(e.message || '复制失败');
      }
    });
  });

  qsa('button[data-adm-toggle]').forEach((b) => b.addEventListener('click', async () => {
    const id = b.getAttribute('data-adm-toggle');
    const key = String(b.getAttribute('data-adm-toggle-key') || '');
    const allowed = ['show', 'sell', 'renew'];
    if (!id || !allowed.includes(key)) return;
    const current = nodePlans.find((item) => String(item.id) === String(id));
    if (!current) return;
    try {
      await apiFetch('/api/v1/admin/node-plans/' + id, {
        method: 'PUT',
        body: { [key]: current[key] ? false : true }
      });
      await renderAdmin(`已更新套餐 #${id} 的 ${key}`);
    } catch (e) {
      await renderAdmin(e.message || '更新失败');
    }
  }));

  qsa('button[data-adm-edit]').forEach(b => b.addEventListener('click', () => openForm('edit', b.getAttribute('data-adm-edit'))));
  qsa('button[data-adm-del]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm('确认删除？')) return;
    try {
      await apiFetch('/api/v1/admin/node-plans/' + b.getAttribute('data-adm-del'), { method: 'DELETE' });
      await renderAdmin();
    } catch (e) {
      await renderAdmin(e.message || '删除失败');
    }
  }));

  const finalizeBtn = qs('#admRefundFinalize');
  if (finalizeBtn) finalizeBtn.addEventListener('click', async () => {
    try {
      await apiFetch('/api/v1/admin/refunds/finalize', { method: 'POST', body: {} });
      await renderAdmin();
    } catch (e) {
      await renderAdmin(e.message || '操作失败');
    }
  });

  qsa('button[data-adm-refund-approve]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm('确认同意退款？')) return;
    const id = b.getAttribute('data-adm-refund-approve');
    try {
      await apiFetch(`/api/v1/admin/refunds/${id}/approve`, { method: 'POST', body: {} });
      await renderAdmin();
    } catch (e) {
      await renderAdmin(e.message || '操作失败');
    }
  }));
  qsa('button[data-adm-refund-deny]').forEach(b => b.addEventListener('click', async () => {
    const id = b.getAttribute('data-adm-refund-deny');
    const reason = prompt('拒绝原因（可选）', '') || '';
    try {
      await apiFetch(`/api/v1/admin/refunds/${id}/deny`, { method: 'POST', body: { reason } });
      await renderAdmin();
    } catch (e) {
      await renderAdmin(e.message || '操作失败');
    }
  }));
}

async function renderProfile(errorText = '') {
  if (!requireAuth()) return;
  const me = await loadMe();

  const info = await apiFetch('/api/v1/user/info');
  const subscribe = await apiFetch('/api/v1/user/getSubscribe');
  const planQuota = await apiFetch('/api/v1/user/plan/quota');
  const traffic = await apiFetch('/api/v1/user/getStat');

  setView(html`
    <div class="grid cols-2">
      <div class="card">
        <h2>账户信息</h2>
        ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
        <div class="kvs">
          <div class="k">邮箱</div><div class="v">${escapeHtml(info?.data?.email || '-')}</div>
          <div class="k">第三方用户名</div><div class="v">${escapeHtml(me?.linux_do_username || '-')}</div>
          <div class="k">订阅链接</div><div class="v"><button class="btn small" id="copySubBtnProfile">复制订阅链接</button></div>
          <div class="k">重置订阅</div><div class="v"><button class="btn small" id="resetSecBtn">重置</button></div>
        </div>
      </div>
      <div class="card">
        <h2>修改密码</h2>
        <div class="grid">
          <div class="field"><label>旧密码</label><input id="oldPwd" type="password"></div>
          <div class="field"><label>新密码</label><input id="newPwd" type="password"></div>
          <div class="row end"><button class="btn" id="chgPwdBtn">修改</button></div>
        </div>
      </div>
    </div>

    <div class="card" style="margin-top: 12px;">
      <h2>统计</h2>
      <div class="muted">未付款订单 / 未关闭工单 / 邀请人数</div>
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">订单</div><div class="v">${traffic?.data?.[0] ?? '-'}</div>
        <div class="k">工单</div><div class="v">${traffic?.data?.[1] ?? '-'}</div>
        <div class="k">邀请</div><div class="v">${traffic?.data?.[2] ?? '-'}</div>
      </div>
    </div>

    <div class="card" style="margin-top:12px;">
      <h2>我的套餐额度使用</h2>
      <div class="kvs" style="margin-top:10px;">
        <div class="k">总额度</div><div class="v">${formatTrafficDisplay(planQuota?.data?.total_allowance_kb || 0, Boolean(planQuota?.data?.has_unlimited_traffic))}</div>
        <div class="k">已使用</div><div class="v">${formatTrafficKb(planQuota?.data?.total_used_kb || 0)}</div>
        <div class="k">剩余额度</div><div class="v">${formatTrafficDisplay(planQuota?.data?.total_remaining_kb || 0, Boolean(planQuota?.data?.has_unlimited_traffic))}</div>
      </div>
      ${renderPlanQuotaTable(planQuota?.data || {})}
    </div>
  `);

  qs('#resetSecBtn').addEventListener('click', async () => {
    if (!confirm('确认重置订阅 token/uuid？')) return;
    try {
      await apiFetch('/api/v1/user/resetSecurity');
      await renderProfile();
    } catch (e) {
      await renderProfile(e.message || '重置失败');
    }
  });

  const copySubBtnProfile = qs('#copySubBtnProfile');
  if (copySubBtnProfile) {
    copySubBtnProfile.addEventListener('click', async () => {
      try {
        await copyText(subscribe?.data?.subscribe_url || '');
        alert('订阅链接已复制');
      } catch (e) {
        await renderProfile(e.message || '复制失败');
      }
    });
  }

  qs('#chgPwdBtn').addEventListener('click', async () => {
    try {
      await apiFetch('/api/v1/user/changePassword', {
        method: 'POST',
        body: { old_password: qs('#oldPwd').value, new_password: qs('#newPwd').value }
      });
      alert('已修改');
    } catch (e) {
      await renderProfile(e.message || '修改失败');
    }
  });
}

async function renderInvite(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const payload = await apiFetch('/api/v1/user/invite/fetch');
  const inviteData = payload?.data || {};
  const codes = asArray(inviteData.codes || payload?.data);
  const availablePlans = asArray(inviteData.available_plans);
  const stat = asArray(inviteData.stat);
  const buildInviteUrl = (code) => buildAppHashUrl('/login', {
    register: '1',
    invite_code: code,
  });
  const getPlanPeriods = (planId) => {
    const normalizedPlanId = Number(planId || 0);
    return asArray(availablePlans.find((item) => Number(item?.id || 0) === normalizedPlanId)?.periods);
  };
  const findPlanName = (planId) => {
    const matched = availablePlans.find((item) => Number(item?.id || 0) === Number(planId || 0));
    return matched?.name || '';
  };
  const findPeriodLabel = (planId, period) => {
    const matched = getPlanPeriods(planId).find((item) => String(item?.value || '') === String(period || ''));
    return matched?.label || String(period || '');
  };

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>邀请</h2>
        <button class="btn" id="newInviteBtn">生成邀请码</button>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid cols-3" style="margin-top:10px;">
        <div class="notice"><strong>邀请注册数：</strong>${stat[0] ?? 0}</div>
        <div class="notice"><strong>已获得佣金：</strong>${stat[1] ?? 0}</div>
        <div class="notice"><strong>可用邀请码：</strong>${codes.length}</div>
      </div>
      <div class="grid cols-2" style="margin-top:12px;">
        <div class="field">
          <label>邀请后自动赠送套餐（可空）</label>
          <select id="inviteAssignedPlanId">
            <option value="">仅注册，不自动赠送套餐</option>
            ${availablePlans.map((plan) => `<option value="${plan.id}">${escapeHtml(plan.name || `套餐 #${plan.id}`)}</option>`).join('')}
          </select>
          <div class="muted">只能选择你自己创建的节点套餐。</div>
        </div>
        <div class="field">
          <label>赠送周期</label>
          <select id="inviteAssignedPeriod" disabled>
            <option value="">请先选择套餐</option>
          </select>
          <div class="muted">套餐和周期必须同时指定。</div>
        </div>
      </div>
      <table class="table" style="margin-top: 10px;">
        <thead><tr><th>邀请码</th><th>赠送套餐</th><th>周期</th><th>邀请链接</th><th>创建时间</th></tr></thead>
        <tbody>
          ${codes.length ? codes.map((c) => {
            const code = c.code || c;
            const inviteUrl = buildInviteUrl(code);
            const assignedPlanId = c.assigned_plan_id || null;
            const assignedPeriod = c.assigned_period || '';
            return `<tr>
              <td class="muted">${escapeHtml(code)}</td>
              <td class="muted">${escapeHtml(c.assigned_plan_name || findPlanName(assignedPlanId) || '-')}</td>
              <td class="muted">${escapeHtml(assignedPeriod ? findPeriodLabel(assignedPlanId, assignedPeriod) : '-')}</td>
              <td>
                <div class="row" style="gap:8px;align-items:center;">
                  <input readonly value="${escapeHtml(inviteUrl)}">
                  <button class="btn small" data-copy-invite="${escapeHtml(inviteUrl)}">复制</button>
                </div>
              </td>
              <td class="muted">${escapeHtml(c.created_at || '-')}</td>
            </tr>`;
          }).join('') : '<tr><td colspan="5" class="muted">暂无可用邀请码</td></tr>'}
        </tbody>
      </table>
    </div>
  `);

  const planSelect = qs('#inviteAssignedPlanId');
  const periodSelect = qs('#inviteAssignedPeriod');
  const syncInvitePeriods = () => {
    if (!periodSelect) return;
    const periods = getPlanPeriods(planSelect?.value);
    if (!periods.length) {
      periodSelect.innerHTML = '<option value="">请先选择套餐</option>';
      periodSelect.disabled = true;
      return;
    }
    periodSelect.innerHTML = periods
      .map((item, index) => `<option value="${escapeHtml(item.value)}" ${index === 0 ? 'selected' : ''}>${escapeHtml(item.label || item.value)}</option>`)
      .join('');
    periodSelect.disabled = false;
  };

  if (planSelect) {
    planSelect.addEventListener('change', syncInvitePeriods);
    syncInvitePeriods();
  }

  qs('#newInviteBtn').addEventListener('click', async () => {
    try {
      const assignedPlanId = String(planSelect?.value || '').trim();
      const assignedPeriod = String(periodSelect?.value || '').trim();
      const body = {};
      if (assignedPlanId || assignedPeriod) {
        if (!assignedPlanId || !assignedPeriod) {
          throw new Error('请同时选择邀请套餐和周期');
        }
        body.assigned_plan_id = assignedPlanId;
        body.assigned_period = assignedPeriod;
      }
      await apiFetch('/api/v1/user/invite/save', { method: 'POST', body });
      await renderInvite();
    } catch (e) {
      await renderInvite(e.message || '生成失败');
    }
  });

  qsa('[data-copy-invite]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      try {
        await copyText(btn.getAttribute('data-copy-invite') || '');
        alert('邀请链接已复制');
      } catch (e) {
        alert(e.message || '复制失败');
      }
    });
  });
}

async function renderNotices(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  const res = await apiFetch('/api/v1/user/notice/fetch');
  const notices = asArray(res?.data);
  const planOptions = asArray(res?.plan_options);
  const manageable = asArray(res?.manageable);
  const canPublish = Boolean(res?.can_publish);

  setView(html`
    <div class="card">
      <h2>可见公告</h2>
      ${errorText ? `<div class="notice error" style="margin-top:10px;">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid" style="margin-top: 10px;">
        ${notices.length ? notices.map((notice) => html`
          <div class="notice">
            <div class="row" style="justify-content: space-between; align-items: center;">
              <div style="font-weight:600;">${escapeHtml(notice.title || '-')}</div>
              <div class="muted">${escapeHtml(formatAnyTimestamp(notice.created_at, '-'))}</div>
            </div>
            ${Array.isArray(notice.tags) && notice.tags.length
              ? `<div class="row" style="margin-top:8px;">${notice.tags.map((tag) => `<span class="pill">${escapeHtml(String(tag))}</span>`).join('')}</div>`
              : ''}
            <div style="margin-top:10px;">${escapeHtml(notice.content || '').replaceAll('\n', '<br>')}</div>
          </div>
        `).join('') : '<div class="notice">当前没有可见公告。</div>'}
      </div>
    </div>

    <div class="grid cols-2" style="margin-top:12px;">
      <div class="card">
        <h2>按套餐发布公告</h2>
        ${canPublish ? `
          <div class="grid">
            <div class="field"><label>标题</label><input id="notice_title" placeholder="例如：今晚 23:00 节点维护"></div>
            <div class="field"><label>标签（逗号分隔，可空）</label><input id="notice_tags" placeholder="维护, 香港, 迁移"></div>
            <div class="field"><label>内容</label><textarea id="notice_content" rows="6" placeholder="只会发给你自己套餐的有效订阅用户"></textarea></div>
            <div class="field">
              <label>目标套餐（至少选择一个）</label>
              <div class="grid">
                ${planOptions.map((plan) => `
                  <label class="row notice-plan-option" style="align-items:center; gap:8px; margin:0;">
                    <input type="checkbox" data-notice-plan value="${plan.id}">
                    <span>${escapeHtml(plan.name || `套餐 #${plan.id}`)}</span>
                  </label>
                `).join('')}
              </div>
            </div>
            <div class="row end">
              <button class="btn primary" id="noticePublishBtn">发布公告</button>
            </div>
          </div>
        ` : '<div class="notice">你还没有自己创建的套餐，当前不能发布定向公告。</div>'}
      </div>

      <div class="card">
        <h2>我发布的公告</h2>
        ${manageable.length ? html`
          <table class="table" style="margin-top:10px;">
            <thead>
              <tr><th>ID</th><th>标题</th><th>套餐范围</th><th>状态</th><th></th></tr>
            </thead>
            <tbody>
              ${manageable.map((notice) => `
                <tr>
                  <td>${notice.id}</td>
                  <td>
                    <div>${escapeHtml(notice.title || '-')}</div>
                    <div class="muted">${escapeHtml(formatAnyTimestamp(notice.created_at, '-'))}</div>
                  </td>
                  <td class="muted">${Array.isArray(notice.target_plan_ids) && notice.target_plan_ids.length ? notice.target_plan_ids.join(', ') : '-'}</td>
                  <td>${notice.show ? '<span class="pill ok">显示中</span>' : '<span class="pill warn">已隐藏</span>'}</td>
                  <td class="row end">
                    <button class="btn small" data-notice-act="toggle" data-id="${notice.id}">${notice.show ? '隐藏' : '显示'}</button>
                    <button class="btn small danger" data-notice-act="delete" data-id="${notice.id}">删除</button>
                  </td>
                </tr>
              `).join('')}
            </tbody>
          </table>
        ` : '<div class="notice" style="margin-top:10px;">你还没有发布过定向公告。</div>'}
      </div>
    </div>
  `);

  const noticePublishBtn = qs('#noticePublishBtn');
  if (noticePublishBtn) {
    noticePublishBtn.addEventListener('click', async () => {
      try {
        const targetPlanIds = qsa('[data-notice-plan]:checked').map((input) => Number(input.value)).filter((value) => Number.isFinite(value) && value > 0);
        if (!targetPlanIds.length) throw new Error('至少选择一个目标套餐');
        const tags = String(qs('#notice_tags')?.value || '')
          .split(',')
          .map((item) => item.trim())
          .filter(Boolean);

        await apiFetch('/api/v1/user/notice', {
          method: 'POST',
          body: {
            title: String(qs('#notice_title')?.value || '').trim(),
            content: String(qs('#notice_content')?.value || '').trim(),
            tags,
            target_plan_ids: targetPlanIds,
          }
        });
        await renderNotices();
      } catch (e) {
        await renderNotices(e.message || '发布公告失败');
      }
    });
  }

  qsa('button[data-notice-act]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const action = String(btn.getAttribute('data-notice-act') || '');
      const id = btn.getAttribute('data-id');
      if (!id) return;
      try {
        if (action === 'toggle') {
          await apiFetch(`/api/v1/user/notice/${id}/toggle`, { method: 'POST', body: {} });
          await renderNotices();
          return;
        }
        if (action === 'delete') {
          if (!confirm('确认删除这条公告？')) return;
          await apiFetch(`/api/v1/user/notice/${id}`, { method: 'DELETE' });
          await renderNotices();
        }
      } catch (e) {
        await renderNotices(e.message || '操作失败');
      }
    });
  });
}

async function renderKnowledge() {
  if (!requireAuth()) return;
  await loadMe();
  const res = await apiFetch('/api/v1/user/knowledge/fetch');
  const items = asArray(res?.data);
  setView(html`
    <div class="card">
      <h2>知识库</h2>
      <div class="grid" style="margin-top: 10px;">
        ${items.map(k => html`
          <div class="notice">
            <div style="font-weight:600;">${k.title || '-'}</div>
            <div class="muted" style="margin-top:6px;">${k.category || ''}</div>
            <div style="margin-top:10px;">${(k.content || '').replaceAll('\n','<br>')}</div>
          </div>
        `).join('')}
      </div>
    </div>
  `);
}

async function renderDownloads(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  setView(html`
    <div class="card">
      <h2>软件下载</h2>
      <div class="muted">选择你的系统，点击按钮就能直接跳转下载。</div>
      ${errorText ? `<div class="notice error" style="margin-top:10px;">${escapeHtml(errorText)}</div>` : ''}
      <div id="downloadsOut" class="grid cols-3" style="margin-top:12px;"></div>
    </div>
  `);

  const container = qs('#downloadsOut');
  if (!container) return;

  try {
    const data = await loadGuestCommConfig();
    const rows = [
      {
        key: 'windows',
        title: 'Windows 客户端',
        version: String(data.windows_version || '').trim(),
        rawUrl: String(data.windows_download_url || '').trim(),
      },
      {
        key: 'macos',
        title: 'macOS 客户端',
        version: String(data.macos_version || '').trim(),
        rawUrl: String(data.macos_download_url || '').trim(),
      },
      {
        key: 'android',
        title: 'Android 客户端',
        version: String(data.android_version || '').trim(),
        rawUrl: String(data.android_download_url || '').trim(),
      },
    ].map((item) => ({
      ...item,
      url: normalizeHttpUrl(item.rawUrl),
    }));

    const hasConfiguredUrl = rows.some((item) => Boolean(item.url));
    container.innerHTML = rows.map((item) => {
      const versionText = item.version || '未填写';
      const invalidUrl = !item.url && item.rawUrl;

      return html`
        <div class="card">
          <h3>${escapeHtml(item.title)}</h3>
          <div class="kvs" style="margin-top:8px;">
            <div class="k">版本</div>
            <div class="v">${escapeHtml(versionText)}</div>
            <div class="k">状态</div>
            <div class="v">${item.url ? '<span class="pill ok">已配置</span>' : '<span class="pill warn">未配置</span>'}</div>
          </div>
          <div style="margin-top:10px;">
            ${item.url
              ? `<input readonly value="${escapeHtml(item.url)}">`
              : `<div class="notice ${invalidUrl ? 'error' : ''}">
                  ${invalidUrl ? '下载地址格式不正确，请联系站长处理。' : '站长还没有配置这个系统的下载地址。'}
                </div>`}
          </div>
          <div class="row" style="margin-top:10px;">
            ${item.url
              ? `<a class="btn primary" href="${escapeHtml(item.url)}" target="_blank" rel="noopener noreferrer">立即下载</a>
                 <button class="btn" data-download-copy="${escapeHtml(item.url)}">复制链接</button>`
              : `<button class="btn" disabled>暂不可下载</button>`}
          </div>
        </div>
      `;
    }).join('');

    if (!hasConfiguredUrl) {
      const notice = document.createElement('div');
      notice.className = 'notice';
      notice.style.gridColumn = '1 / -1';
      notice.textContent = '当前还没有可用下载地址，请先让站长在后台填写。';
      container.appendChild(notice);
    }

    qsa('[data-download-copy]', container).forEach((btn) => {
      btn.addEventListener('click', async () => {
        const value = btn.getAttribute('data-download-copy') || '';
        try {
          await copyText(value);
          alert('下载链接已复制');
        } catch (e) {
          alert(e.message || '复制失败');
        }
      });
    });
  } catch (e) {
    container.innerHTML = `<div class="notice error">${escapeHtml(e.message || '加载失败')}</div>`;
  }
}

function formatAnyTimestamp(rawValue, fallback = '-') {
  if (rawValue === null || rawValue === undefined || rawValue === '') return fallback;
  if (typeof rawValue === 'number' && Number.isFinite(rawValue)) return formatTs(rawValue);

  const text = String(rawValue).trim();
  if (!text) return fallback;

  const asInt = Number(text);
  if (Number.isFinite(asInt) && text.match(/^\d+$/)) {
    return formatTs(asInt);
  }

  const parsed = new Date(text);
  if (!Number.isNaN(parsed.getTime())) {
    return parsed.toLocaleString();
  }

  return text;
}

function periodKeyLabel(periodKey) {
  const map = Object.fromEntries(PLAN_PRICE_FIELDS.map((item) => [item.key, item.label]));
  return map[String(periodKey || '')] || String(periodKey || '-');
}

function renderCouponCheckSummary(couponData) {
  if (!couponData || typeof couponData !== 'object') {
    return '<div class="notice">未获取到优惠券信息。</div>';
  }

  const type = Number(couponData.type || 0);
  let discountText = '未识别';
  if (type === 1) {
    discountText = `立减 ${`¥${(Number(couponData.value || 0) / 100).toFixed(2)}`}`;
  } else if (type === 2) {
    discountText = `${Number(couponData.value || 0)}% 折扣`;
  }

  const limitPlanIds = Array.isArray(couponData.limit_plan_ids) ? couponData.limit_plan_ids : [];
  const limitPeriods = Array.isArray(couponData.limit_period) ? couponData.limit_period : [];

  return html`
    <div class="kvs">
      <div class="k">优惠券名称</div><div class="v">${escapeHtml(couponData.name || '-')}</div>
      <div class="k">优惠码</div><div class="v">${escapeHtml(couponData.code || '-')}</div>
      <div class="k">优惠内容</div><div class="v">${escapeHtml(discountText)}</div>
      <div class="k">开始时间</div><div class="v">${escapeHtml(formatAnyTimestamp(couponData.started_at, '未设置'))}</div>
      <div class="k">结束时间</div><div class="v">${escapeHtml(formatAnyTimestamp(couponData.ended_at, '未设置'))}</div>
      <div class="k">剩余总次数</div><div class="v">${couponData.limit_use === null ? '不限' : String(couponData.limit_use)}</div>
      <div class="k">每用户可用</div><div class="v">${couponData.limit_use_with_user === null ? '不限' : String(couponData.limit_use_with_user) + ' 次'}</div>
      <div class="k">适用套餐</div><div class="v">${limitPlanIds.length ? limitPlanIds.join(', ') : '全部套餐'}</div>
      <div class="k">适用周期</div><div class="v">${limitPeriods.length ? limitPeriods.map(periodKeyLabel).join(' / ') : '全部周期'}</div>
    </div>
  `;
}

function renderGiftRewardRows(rewardData) {
  if (!rewardData || typeof rewardData !== 'object') {
    return '<div class="muted">无奖励明细</div>';
  }

  const entries = Object.entries(rewardData);
  if (!entries.length) return '<div class="muted">无奖励明细</div>';

  const labelMap = {
    balance: '余额奖励',
    transfer_enable: '流量奖励',
    device_limit: '设备数奖励',
    expire_days: '有效期延长(天)',
    plan_id: '赠送套餐 ID',
    plan_validity_days: '套餐有效期(天)',
    reset_package: '赠送流量重置',
    invite_reward_rate: '邀请奖励比例',
  };

  const valueText = (key, value) => {
    if (key === 'balance') return `¥${(Number(value || 0) / 100).toFixed(2)}`;
    if (typeof value === 'boolean') return value ? '是' : '否';
    if (value === null || value === undefined || value === '') return '-';
    return String(value);
  };

  return html`
    <div class="kvs">
      ${entries.map(([key, value]) => `
        <div class="k">${escapeHtml(labelMap[key] || key)}</div>
        <div class="v">${escapeHtml(valueText(key, value))}</div>
      `).join('')}
    </div>
  `;
}

function renderGiftCheckSummary(data) {
  const canRedeem = Boolean(data?.can_redeem);
  const statusPill = canRedeem
    ? '<span class="pill ok">可兑换</span>'
    : '<span class="pill warn">暂不可兑换</span>';
  const reason = data?.reason ? String(data.reason) : '';
  const info = data?.code_info || {};
  const template = info?.template || {};

  return html`
    <div class="kvs">
      <div class="k">兑换状态</div><div class="v">${statusPill}</div>
      ${reason ? `<div class="k">说明</div><div class="v">${escapeHtml(reason)}</div>` : ''}
      <div class="k">礼品卡类型</div><div class="v">${escapeHtml(template?.type_name || template?.name || '-')}</div>
      <div class="k">卡片状态</div><div class="v">${escapeHtml(info?.status_name || '-')}</div>
      <div class="k">过期时间</div><div class="v">${escapeHtml(formatAnyTimestamp(info?.expires_at, '不限'))}</div>
      <div class="k">可用次数</div><div class="v">${escapeHtml(`${info?.usage_count || 0} / ${info?.max_usage || 0}`)}</div>
    </div>
    <div class="card" style="margin-top:10px;">
      <h3>奖励预览</h3>
      ${renderGiftRewardRows(data?.reward_preview || {})}
    </div>
  `;
}

function renderGiftRedeemSummary(data) {
  return html`
    <div class="notice ok">${escapeHtml(data?.message || '兑换成功')}</div>
    <div class="card" style="margin-top:10px;">
      <h3>本次到账</h3>
      ${renderGiftRewardRows(data?.rewards || {})}
    </div>
    ${data?.invite_rewards
      ? `<div class="card" style="margin-top:10px;">
          <h3>邀请奖励</h3>
          ${renderGiftRewardRows(data.invite_rewards)}
        </div>`
      : ''}
  `;
}

function renderNodeTrafficSummary(items) {
  const rows = Array.isArray(items) ? items : [];
  if (!rows.length) {
    return '<div class="notice">今天还没有流量记录。</div>';
  }

  return html`
    <table class="table">
      <thead>
        <tr><th>节点</th><th>状态</th><th>今日上行</th><th>今日下行</th><th>节点流量上限</th><th>TCPing</th></tr>
      </thead>
      <tbody>
        ${rows.map((item) => {
          const today = item?.today || {};
          const up = Number(today.upload || 0);
          const down = Number(today.download || 0);
          return `
            <tr>
              <td>
                <div>${escapeHtml(item?.node_name || ('节点 #' + (item?.node_id || '-')))}</div>
                <div class="muted">${escapeHtml(item?.location_name || '-')} · ${escapeHtml(protocolDisplayName(item?.protocol || ''))}</div>
                <div class="muted">套餐扣费倍率 x${escapeHtml(String(Number(item?.traffic_multiplier || 1).toFixed(2)))}</div>
              </td>
              <td>${pillStatus(item || {})}</td>
              <td class="muted">${escapeHtml(formatTrafficKb(up))}</td>
              <td class="muted">${escapeHtml(formatTrafficKb(down))}</td>
              <td>${renderNodeTrafficCell(item)}</td>
              <td>${renderNodeTcpingCell(item)}</td>
            </tr>
          `;
        }).join('')}
      </tbody>
    </table>
  `;
}

function renderTrafficUsageLogs(logs) {
  const rows = Array.isArray(logs) ? logs : [];
  if (!rows.length) {
    return '<div class="notice">最近还没有流量使用记录。</div>';
  }

  return html`
    <table class="table">
      <thead>
        <tr><th>时间</th><th>节点</th><th>原始流量</th><th>计费流量</th><th>倍率</th><th>来源</th></tr>
      </thead>
      <tbody>
        ${rows.map((log) => `
          <tr>
            <td class="muted">${escapeHtml(formatTs(log.recorded_at))}</td>
            <td>
              <div>${escapeHtml(log.node_name || `节点 #${log.node_id || '-'}`)}</div>
              <div class="muted">${escapeHtml(log.location_name || '-')} · ${escapeHtml(protocolDisplayName(log.protocol || ''))}</div>
            </td>
            <td class="muted">${escapeHtml(formatTrafficKb(log.raw_traffic_kb || 0))}</td>
            <td class="muted">${escapeHtml(formatTrafficKb(log.billed_traffic_kb || 0))}</td>
            <td><span class="pill">x${escapeHtml(String(Number(log.multiplier_snapshot || 1).toFixed(2)))}</span></td>
            <td class="muted">${escapeHtml(log.source || '-')}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;
}

async function renderTools(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();
  setView(html`
    <div class="grid cols-2">
      <div class="card">
        <h2>优惠券</h2>
        ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
        <div class="row">
          <input id="couponCode" placeholder="请输入优惠码">
          <button class="btn" id="couponCheckBtn">查询</button>
        </div>
        <div id="couponOut" style="margin-top:10px;"></div>
      </div>
      <div class="card">
        <h2>礼品卡</h2>
        <div class="row">
          <input id="giftCode" placeholder="请输入礼品卡兑换码">
          <button class="btn" id="giftCheckBtn">查询</button>
          <button class="btn primary" id="giftRedeemBtn">兑换</button>
        </div>
        <div id="giftOut" style="margin-top:10px;"></div>
      </div>
    </div>

    <div class="card" style="margin-top:12px;">
      <h2>节点流量（我的）</h2>
      <div class="muted">来自共享平台节点系统</div>
      <div id="nodeTrafficOut" style="margin-top:10px;"></div>
    </div>

    <div class="card" style="margin-top:12px;">
      <h2>流量使用记录</h2>
      <div class="muted">这里会同时展示原始流量和套餐实际扣费流量。倍率只影响套餐消耗，不影响提供节点的已用总流量显示。</div>
      <div id="trafficUsageLogOut" style="margin-top:10px;"></div>
    </div>
  `);

  qs('#couponCheckBtn').addEventListener('click', async () => {
    try {
      const res = await apiFetch('/api/v1/user/coupon/check', { method: 'POST', body: { code: qs('#couponCode').value.trim() } });
      qs('#couponOut').innerHTML = renderCouponCheckSummary(res?.data || {});
    } catch (e) {
      qs('#couponOut').innerHTML = `<div class="notice error">${escapeHtml(e.message || '失败')}</div>`;
    }
  });

  qs('#giftCheckBtn').addEventListener('click', async () => {
    try {
      const res = await apiFetch('/api/v1/user/gift-card/check', { method: 'POST', body: { code: qs('#giftCode').value.trim() } });
      qs('#giftOut').innerHTML = renderGiftCheckSummary(res?.data || {});
    } catch (e) {
      qs('#giftOut').innerHTML = `<div class="notice error">${escapeHtml(e.message || '失败')}</div>`;
    }
  });

  qs('#giftRedeemBtn').addEventListener('click', async () => {
    try {
      const res = await apiFetch('/api/v1/user/gift-card/redeem', { method: 'POST', body: { code: qs('#giftCode').value.trim() } });
      qs('#giftOut').innerHTML = renderGiftRedeemSummary(res?.data || {});
    } catch (e) {
      qs('#giftOut').innerHTML = `<div class="notice error">${escapeHtml(e.message || '失败')}</div>`;
    }
  });

  try {
    const traffic = await apiFetch('/api/v1/user/node-traffic');
    qs('#nodeTrafficOut').innerHTML = renderNodeTrafficSummary(asArray(traffic?.data));
  } catch (e) {
    qs('#nodeTrafficOut').innerHTML = `<div class="notice error">${escapeHtml(e.message || '失败')}</div>`;
  }

  try {
    const logs = await apiFetch('/api/v1/user/traffic-usage-logs?days=7&limit=100');
    qs('#trafficUsageLogOut').innerHTML = renderTrafficUsageLogs(asArray(logs?.data));
  } catch (e) {
    qs('#trafficUsageLogOut').innerHTML = `<div class="notice error">${escapeHtml(e.message || '失败')}</div>`;
  }
}

async function renderPaymentProfile(errorText = '') {
  if (!requireAuth()) return;
  const me = await loadMe();

  let mine = null;
  try {
    const res = await apiFetch('/api/v1/user/payment-profiles/epay');
    mine = res?.data || null;
  } catch (e) {
    mine = null;
  }

  let sponsor = null;
  let sponsorErr = '';
  if (me?.is_super_admin) {
    try {
      const res = await apiFetch('/api/v1/admin/sponsor-epay');
      sponsor = res?.data || null;
    } catch (e) {
      sponsorErr = e.message || '加载失败';
    }
  }

  setView(html`
    <div class="grid ${me?.is_super_admin ? 'cols-2' : ''}">
      <div class="card">
        <h2>我的收款（EPay 协议）</h2>
        ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
        <div class="grid">
          <div class="field"><label>网关 URL</label><input id="pp_url" value="${escapeHtml(mine?.url || 'https://credit.linux.do/epay')}"></div>
          <div class="field"><label>提交路径</label><input id="pp_submit" value="${escapeHtml(mine?.submit_path || '/pay/submit.php')}"></div>
          <div class="field"><label>商户号 PID</label><input id="pp_pid" value="${escapeHtml(mine?.pid || '')}"></div>
          <div class="field"><label>商户密钥（加密保存）</label><input id="pp_key" type="password" placeholder="${mine?.has_key ? '已设置(重新输入可覆盖)' : ''}"></div>
          <div class="field"><label>提交方式</label>
            <select id="pp_post">
              <option value="1">POST</option>
              <option value="0">GET</option>
            </select>
          </div>
          <div class="field"><label>站点名称（可选）</label><input id="pp_site" value="${escapeHtml(mine?.sitename || '')}"></div>
          <div class="field"><label>设备标识（可选）</label><input id="pp_device" value="${escapeHtml(mine?.device || '')}"></div>
          <div class="row end"><button class="btn primary" id="pp_save">保存</button></div>
        </div>
        <div class="muted" style="margin-top:10px;">
          用户购买你发布的“节点套餐”时，会优先使用你的商户号和密钥进行支付（对应你的节点收益）。
        </div>
      </div>

      ${me?.is_super_admin ? html`
        <div class="card">
          <h2>赞助收款（超管）</h2>
          ${sponsorErr ? `<div class="notice error">${escapeHtml(sponsorErr)}</div>` : ''}
          <div class="grid">
            <div class="field"><label>网关 URL</label><input id="sp_url" value="${escapeHtml(sponsor?.url || 'https://credit.linux.do/epay')}"></div>
            <div class="field"><label>提交路径</label><input id="sp_submit" value="${escapeHtml(sponsor?.submit_path || '/pay/submit.php')}"></div>
            <div class="field"><label>商户号 PID</label><input id="sp_pid" value="${escapeHtml(sponsor?.pid || '')}"></div>
            <div class="field"><label>商户密钥</label><input id="sp_key" type="password" placeholder="${sponsor?.has_key ? '已设置(重新输入可覆盖)' : ''}"></div>
            <div class="field"><label>提交方式</label>
              <select id="sp_post">
                <option value="1">POST</option>
                <option value="0">GET</option>
              </select>
            </div>
            <div class="field"><label>站点名称（可选）</label><input id="sp_site" value="${escapeHtml(sponsor?.sitename || '')}"></div>
            <div class="field"><label>设备标识（可选）</label><input id="sp_device" value="${escapeHtml(sponsor?.device || '')}"></div>
            <div class="row end"><button class="btn" id="sp_save">保存</button></div>
          </div>
          <div class="muted" style="margin-top:10px;">
            对于节点套餐订单，用户可在支付时选择 <code>pay_to=sponsor</code>，使用此处配置的商户号和密钥付款（赞助超管）。
          </div>
        </div>
      ` : ''}
    </div>
  `);

  if (qs('#pp_post')) qs('#pp_post').value = String(mine?.use_post ? 1 : 0);
  if (qs('#sp_post') && sponsor) qs('#sp_post').value = String(sponsor?.use_post ? 1 : 0);

  qs('#pp_save').addEventListener('click', async () => {
    try {
      await apiFetch('/api/v1/user/payment-profiles/epay', {
        method: 'PUT',
        body: {
          url: qs('#pp_url').value.trim(),
          submit_path: qs('#pp_submit').value.trim(),
          pid: qs('#pp_pid').value.trim(),
          key: qs('#pp_key').value || '',
          use_post: qs('#pp_post').value === '1',
          sitename: qs('#pp_site').value,
          device: qs('#pp_device').value
        }
      });
      alert('已保存');
      location.hash = '#/payment';
    } catch (e) {
      renderPaymentProfile(e.message || '保存失败');
    }
  });

  const spSave = qs('#sp_save');
  if (spSave) {
    spSave.addEventListener('click', async () => {
      try {
        await apiFetch('/api/v1/admin/sponsor-epay', {
          method: 'PUT',
          body: {
            url: qs('#sp_url').value.trim(),
            submit_path: qs('#sp_submit').value.trim(),
            pid: qs('#sp_pid').value.trim(),
            key: qs('#sp_key').value || '',
            use_post: qs('#sp_post').value === '1',
            sitename: qs('#sp_site').value,
            device: qs('#sp_device').value
          }
        });
        alert('已保存');
        location.hash = '#/payment';
      } catch (e) {
        renderPaymentProfile(e.message || '保存失败');
      }
    });
  }
}

async function renderSponsor(errorText = '') {
  if (!requireAuth()) return;
  await loadMe();

  let methods = [];
  try {
    const m = await apiFetch('/api/v1/user/sponsor/methods');
    methods = asArray(m?.data);
  } catch (e) {
    methods = [];
  }

  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>赞助</h2>
        <div class="muted">独立于套餐购买</div>
      </div>
      ${errorText ? `<div class="notice error">${escapeHtml(errorText)}</div>` : ''}
      <div class="grid cols-2" style="margin-top: 10px;">
        <div class="field"><label>金额（元）</label><input id="sp_amount" type="number" step="0.01" value="10"></div>
        <div class="field"><label>支付方式（EPay）</label>
          <select id="sp_method">
            ${methods.map(m => `<option value="${m.id}">${escapeHtml(m.name)}</option>`).join('')}
          </select>
        </div>
      </div>
      <div class="row end" style="margin-top: 12px;">
        <button class="btn primary" id="sp_create">发起赞助</button>
      </div>
      <div id="sp_out" style="margin-top: 12px;"></div>
    </div>
  `);

  qs('#sp_create').addEventListener('click', async () => {
    try {
      const amount = Number(qs('#sp_amount').value);
      const methodId = Number(qs('#sp_method').value);
      const created = await apiFetch('/api/v1/user/sponsor', { method: 'POST', body: { amount } });
      const tradeNo = created?.data?.trade_no;
      const out = await apiFetch('/api/v1/user/sponsor/checkout', {
        method: 'POST',
        body: { trade_no: tradeNo, method: methodId }
      });

      const box = qs('#sp_out');
      if (out?.type === 1 && typeof out?.data === 'string' && out.data.includes('<form')) {
        submitSafeEpayPostForm(box, out.data, '已生成支付表单，正在跳转...');
        return;
      }
      if (out?.type === 1 && typeof out?.data === 'string') {
        const paymentUrl = normalizeHttpUrl(out.data);
        if (!paymentUrl) throw new Error('支付网关返回了无效链接');
        const notice = document.createElement('div');
        notice.className = 'notice';
        notice.append(document.createTextNode('支付链接：'));
        const link = document.createElement('a');
        link.href = paymentUrl;
        link.target = '_blank';
        link.rel = 'noopener noreferrer';
        link.textContent = paymentUrl;
        notice.append(link);
        box.replaceChildren(notice);
        window.location.href = paymentUrl;
        return;
      }
      const pre = document.createElement('pre');
      pre.style.whiteSpace = 'pre-wrap';
      pre.style.color = 'var(--muted)';
      pre.textContent = JSON.stringify(out, null, 2);
      box.replaceChildren(pre);
    } catch (e) {
      renderSponsor(e.message || '发起失败');
    }
  });
}

async function renderSponsorDetail(tradeNo) {
  if (!requireAuth()) return;
  await loadMe();
  const res = await apiFetch('/api/v1/user/sponsor/' + encodeURIComponent(tradeNo));
  const d = res?.data || {};
  setView(html`
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center;">
        <h2>赞助订单</h2>
        <a class="btn" href="#/sponsor">返回</a>
      </div>
      <div class="kvs" style="margin-top: 10px;">
        <div class="k">trade_no</div><div class="v">${d.trade_no}</div>
        <div class="k">金额(分)</div><div class="v">${d.total_amount}</div>
        <div class="k">状态</div><div class="v"><span class="pill">${d.status}</span></div>
        <div class="k">paid_at</div><div class="v">${d.paid_at || '-'}</div>
      </div>
      <div class="muted" style="margin-top:10px;">支付完成后，网关会回调并将状态置为已完成。</div>
    </div>
  `);
}

async function router() {
  clearViewCleanup();
  syncAuthNav();
  const route = parseHashRoute(location.hash || '#/dashboard');
  const hash = route.rawHash;
  setActiveNav();
  const [page, a, b] = route.segments;
  const commandCenterMode = page === 'admin' && a === 'command-center';
  const adminTemplateMode = page === 'admin' && a === 'template';
  setAppChromeMode(commandCenterMode);
  setContainerWideMode(page === 'admin' || adminTemplateMode);

  if (!store.auth && page !== 'login') {
    setPostLoginRedirect(hash);
    location.hash = '#/login';
    return;
  }

  if (store.auth && !store.me) {
    await loadMe().catch(() => {});
  }

  if (page === 'login') {
    prewarmLoginCaptcha();
    ensurePowProofForMode(route.query.register === '1' ? 'register' : 'login').catch(() => {});
    if (store.auth) {
      const redirectHash = consumePostLoginRedirect();
      location.hash = redirectHash || '#/dashboard';
      return;
    }
    return renderLogin('', route.query.register === '1' ? 'register' : 'login', {
      inviteCode: route.query.invite_code || '',
    });
  }
  if (page === 'dashboard' || !page) return renderDashboard();
  if (page === 'plans') return renderPlans();
  if (page === 'plan-link' && a) {
    let token = a;
    try {
      token = decodeURIComponent(a);
    } catch (_) {
      token = a;
    }
    return renderPlans(token);
  }
  if (page === 'orders') return renderOrders();
  if (page === 'order' && a) return renderOrderDetail(a);
  if (page === 'refunds') return renderRefunds();
  if (page === 'refund' && a) return renderRefundDetail(a);
  const disputeEnabled = Boolean(store.me?.refund_dispute_enable);
  if (page === 'refund-votes') {
    if (disputeEnabled) return renderRefundVoteList();
    if (hash !== '#/refunds') {
      location.hash = '#/refunds';
    }
    return;
  }
  if (page === 'refund-vote' && a) {
    if (disputeEnabled) return renderRefundVoteDetail(a);
    if (hash !== '#/refunds') {
      location.hash = '#/refunds';
    }
    return;
  }
  if (page === 'tickets' && a === 'new') return renderNewTicket();
  if (page === 'tickets') return renderTickets();
  if (page === 'ticket' && a) return renderTicketDetail(a);
  if (page === 'nodes' && a === 'new') return renderNewNode();
  if (page === 'nodes' && a === 'edit' && b) return renderEditNode(b);
  if (page === 'nodes' && a && b === 'access') return renderNodeAccess(a);
  if (page === 'nodes' && a && b === 'audit-rules') return renderNodeAuditRules(a);
  if (page === 'nodes' && a && b === 'monitor') return renderNodeMonitor(a, route.query.hours || 24);
  if (page === 'nodes') return renderNodes();
  if (page === 'node-plans' && a === 'new') return renderNodePlanForm('new');
  if (page === 'node-plans' && a === 'edit' && b) return renderNodePlanForm('edit', b);
  if (page === 'node-plans') return renderNodePlans();
  if (page === 'node-admin' && a === 'node' && b) return renderNodeAdminNodeUsers(b);
  if (page === 'node-admin' && a === 'ticket' && b) return renderNodeAdminTicket(b);
  if (page === 'node-admin' && a === 'refunds') return renderNodeAdminRefunds();
  if (page === 'node-admin' && a === 'refund' && b) return renderNodeAdminRefundDetail(b);
  if (page === 'node-admin') return renderNodeAdmin();
  if (page === 'audit') return renderAudit();
  if (page === 'admin' && a === 'command-center') return renderAdminCommandCenter();
  if (page === 'admin' && a === 'template' && b) return renderAdminTemplateEditor(b);
  if (page === 'admin' && a === 'template') return renderAdminTemplateEditor();
  if (page === 'admin') return renderAdmin('', a);
  if (page === 'profile') return renderProfile();
  if (page === 'invite') return renderInvite();
  if (page === 'notices') return renderNotices();
  if (page === 'knowledge') return renderKnowledge();
  if (page === 'downloads') return renderDownloads();
  if (page === 'tools') return renderTools();
  if (page === 'payment') return renderPaymentProfile();
  if (page === 'sponsor' && a) return renderSponsorDetail(a);
  if (page === 'sponsor') return renderSponsor();

  setView(`<div class="notice error">未知页面：${escapeHtml(hash)}</div>`);
}

function setNavOpen(open) {
  const enabled = Boolean(open);
  document.body.classList.toggle('nav-open', enabled);
  if (NAV_TOGGLE) {
    NAV_TOGGLE.setAttribute('aria-expanded', enabled ? 'true' : 'false');
    NAV_TOGGLE.setAttribute('aria-label', enabled ? '关闭导航' : '打开导航');
  }
  if (NAV_SCRIM) NAV_SCRIM.tabIndex = enabled ? 0 : -1;
}

if (NAV_TOGGLE) {
  NAV_TOGGLE.addEventListener('click', () => {
    setNavOpen(!document.body.classList.contains('nav-open'));
  });
}

if (NAV_SCRIM) {
  NAV_SCRIM.addEventListener('click', () => setNavOpen(false));
}

NAV_LINKS.forEach((link) => {
  link.addEventListener('click', () => setNavOpen(false));
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape' && document.body.classList.contains('nav-open')) {
    setNavOpen(false);
    NAV_TOGGLE?.focus();
  }
});

logoutBtn.addEventListener('click', (e) => {
  if (!store.auth) return;
  clearAuthState();
  syncAuthNav();
});

window.addEventListener('storage', (event) => {
  if (!SHARED_AUTH_STORAGE_KEYS.includes(event.key || '') && event.key !== 'me') {
    return;
  }

  syncAuthNav();
  router().catch((e) => {
    if (e?.handled) return;
    setView(`<div class="notice error">${escapeHtml(e.message || '发生错误')}</div>`);
  });
});

window.addEventListener('hashchange', () => {
  setNavOpen(false);
  router().catch(e => {
    if (e?.handled) return;
    setView(`<div class="notice error">${escapeHtml(e.message || '发生错误')}</div>`);
  });
});

// First load
router().catch(e => {
  if (e?.handled) return;
  setView(`<div class="notice error">${escapeHtml(e.message || '发生错误')}</div>`);
});
