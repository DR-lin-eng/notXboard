(function (window) {
    'use strict';

    if (!window.echarts) {
        return;
    }

    var COUNTRY_COORDS = Object.freeze({
        CA: [-106.3468, 56.1304],
        US: [-95.7129, 37.0902],
        MX: [-102.5528, 23.6345],
        BR: [-51.9253, -14.235],
        AR: [-63.6167, -38.4161],
        GB: [-3.436, 55.3781],
        IE: [-8.2439, 53.4129],
        PT: [-8.2245, 39.3999],
        ES: [-3.7492, 40.4637],
        FR: [2.2137, 46.2276],
        DE: [10.4515, 51.1657],
        NL: [5.2913, 52.1326],
        IT: [12.5674, 41.8719],
        RU: [105.3188, 61.524],
        TR: [35.2433, 38.9637],
        EG: [30.8025, 26.8206],
        ZA: [22.9375, -30.5595],
        AE: [53.8478, 23.4241],
        SA: [45.0792, 23.8859],
        IN: [78.9629, 20.5937],
        PK: [69.3451, 30.3753],
        KZ: [66.9237, 48.0196],
        CN: [104.1954, 35.8617],
        'CN-BJ': [116.4074, 39.9042],
        'CN-TJ': [117.2000, 39.1333],
        'CN-HE': [114.5149, 38.0428],
        'CN-SX': [112.5492, 37.8706],
        'CN-NM': [111.7492, 40.8426],
        'CN-LN': [123.4315, 41.8057],
        'CN-JL': [125.3235, 43.8171],
        'CN-HL': [126.6424, 45.7560],
        'CN-SH': [121.4737, 31.2304],
        'CN-JS': [118.7969, 32.0603],
        'CN-ZJ': [120.1536, 30.2655],
        'CN-AH': [117.2272, 31.8206],
        'CN-FJ': [119.2965, 26.0745],
        'CN-JX': [115.8582, 28.6820],
        'CN-SD': [117.1201, 36.6512],
        'CN-HA': [113.6254, 34.7466],
        'CN-HB': [114.3054, 30.5931],
        'CN-HN': [112.9388, 28.2282],
        'CN-GD': [113.2644, 23.1291],
        'CN-GX': [108.3200, 22.8240],
        'CN-HI': [110.3312, 20.0310],
        'CN-CQ': [106.5516, 29.5630],
        'CN-SC': [104.0665, 30.5728],
        'CN-GZ': [106.6302, 26.6470],
        'CN-YN': [102.7123, 25.0406],
        'CN-XZ': [91.1322, 29.6604],
        'CN-SN': [108.9398, 34.3416],
        'CN-GS': [103.8343, 36.0611],
        'CN-QH': [101.7782, 36.6171],
        'CN-NX': [106.2782, 38.4664],
        'CN-XJ': [87.6177, 43.7928],
        HK: [114.1694, 22.3193],
        MO: [113.5439, 22.1987],
        TW: [120.9605, 23.6978],
        KR: [127.7669, 35.9078],
        JP: [138.2529, 36.2048],
        SG: [103.8198, 1.3521],
        MY: [101.9758, 4.2105],
        TH: [100.9925, 15.87],
        VN: [108.2772, 14.0583],
        PH: [121.774, 12.8797],
        ID: [113.9213, -0.7893],
        AU: [133.7751, -25.2744],
        NZ: [174.886, -40.9006],
        CL: [-71.543, -35.6751],
        CO: [-74.2973, 4.5709],
        PE: [-75.0152, -9.19],
        SE: [18.6435, 60.1282],
        NO: [8.4689, 60.472],
        FI: [25.7482, 61.9241],
        PL: [19.1451, 51.9194],
        UA: [31.1656, 48.3794],
        CH: [8.2275, 46.8182],
        AT: [14.5501, 47.5162],
        BE: [4.4699, 50.5039],
        DK: [9.5018, 56.2639],
        RO: [24.9668, 45.9432],
        HU: [19.5033, 47.1625]
    });

    var LOCATION_ALIASES = Object.freeze([
        ['BEIJING', 'CN-BJ'],
        ['北京市', 'CN-BJ'],
        ['北京', 'CN-BJ'],
        ['TIANJIN', 'CN-TJ'],
        ['天津市', 'CN-TJ'],
        ['天津', 'CN-TJ'],
        ['HEBEI', 'CN-HE'],
        ['河北省', 'CN-HE'],
        ['河北', 'CN-HE'],
        ['SHANXI', 'CN-SX'],
        ['山西省', 'CN-SX'],
        ['山西', 'CN-SX'],
        ['INNER MONGOLIA', 'CN-NM'],
        ['NEIMENGGU', 'CN-NM'],
        ['内蒙古自治区', 'CN-NM'],
        ['内蒙古', 'CN-NM'],
        ['LIAONING', 'CN-LN'],
        ['辽宁省', 'CN-LN'],
        ['辽宁', 'CN-LN'],
        ['JILIN', 'CN-JL'],
        ['吉林省', 'CN-JL'],
        ['吉林', 'CN-JL'],
        ['HEILONGJIANG', 'CN-HL'],
        ['黑龙江省', 'CN-HL'],
        ['黑龙江', 'CN-HL'],
        ['SHANGHAI', 'CN-SH'],
        ['上海市', 'CN-SH'],
        ['上海', 'CN-SH'],
        ['JIANGSU', 'CN-JS'],
        ['江苏省', 'CN-JS'],
        ['江苏', 'CN-JS'],
        ['ZHEJIANG', 'CN-ZJ'],
        ['浙江省', 'CN-ZJ'],
        ['浙江', 'CN-ZJ'],
        ['ANHUI', 'CN-AH'],
        ['安徽省', 'CN-AH'],
        ['安徽', 'CN-AH'],
        ['FUJIAN', 'CN-FJ'],
        ['福建省', 'CN-FJ'],
        ['福建', 'CN-FJ'],
        ['JIANGXI', 'CN-JX'],
        ['江西省', 'CN-JX'],
        ['江西', 'CN-JX'],
        ['SHANDONG', 'CN-SD'],
        ['山东省', 'CN-SD'],
        ['山东', 'CN-SD'],
        ['HENAN', 'CN-HA'],
        ['河南省', 'CN-HA'],
        ['河南', 'CN-HA'],
        ['HUBEI', 'CN-HB'],
        ['湖北省', 'CN-HB'],
        ['湖北', 'CN-HB'],
        ['HUNAN', 'CN-HN'],
        ['湖南省', 'CN-HN'],
        ['湖南', 'CN-HN'],
        ['GUANGDONG', 'CN-GD'],
        ['广东省', 'CN-GD'],
        ['广东', 'CN-GD'],
        ['GUANGXI', 'CN-GX'],
        ['广西壮族自治区', 'CN-GX'],
        ['广西', 'CN-GX'],
        ['HAINAN', 'CN-HI'],
        ['海南省', 'CN-HI'],
        ['海南', 'CN-HI'],
        ['CHONGQING', 'CN-CQ'],
        ['重庆市', 'CN-CQ'],
        ['重庆', 'CN-CQ'],
        ['SICHUAN', 'CN-SC'],
        ['四川省', 'CN-SC'],
        ['四川', 'CN-SC'],
        ['GUIZHOU', 'CN-GZ'],
        ['贵州省', 'CN-GZ'],
        ['贵州', 'CN-GZ'],
        ['YUNNAN', 'CN-YN'],
        ['云南省', 'CN-YN'],
        ['云南', 'CN-YN'],
        ['TIBET', 'CN-XZ'],
        ['XIZANG', 'CN-XZ'],
        ['西藏自治区', 'CN-XZ'],
        ['西藏', 'CN-XZ'],
        ['SHAANXI', 'CN-SN'],
        ['陕西省', 'CN-SN'],
        ['陕西', 'CN-SN'],
        ['GANSU', 'CN-GS'],
        ['甘肃省', 'CN-GS'],
        ['甘肃', 'CN-GS'],
        ['QINGHAI', 'CN-QH'],
        ['青海省', 'CN-QH'],
        ['青海', 'CN-QH'],
        ['NINGXIA', 'CN-NX'],
        ['宁夏回族自治区', 'CN-NX'],
        ['宁夏', 'CN-NX'],
        ['XINJIANG', 'CN-XJ'],
        ['新疆维吾尔自治区', 'CN-XJ'],
        ['新疆', 'CN-XJ'],
        ['GUANGZHOU', 'CN-GD'],
        ['广州', 'CN-GD'],
        ['SHENZHEN', 'CN-GD'],
        ['深圳', 'CN-GD'],
        ['HANGZHOU', 'CN-ZJ'],
        ['杭州', 'CN-ZJ'],
        ['NANJING', 'CN-JS'],
        ['南京', 'CN-JS'],
        ['FUZHOU', 'CN-FJ'],
        ['福州', 'CN-FJ'],
        ['CHENGDU', 'CN-SC'],
        ['成都', 'CN-SC'],
        ['WUHAN', 'CN-HB'],
        ['武汉', 'CN-HB'],
        ['XIAN', 'CN-SN'],
        ['西安', 'CN-SN'],
        ['URUMQI', 'CN-XJ'],
        ['乌鲁木齐', 'CN-XJ'],
        ['HONGKONG', 'HK'],
        ['HONG KONG', 'HK'],
        ['香港特别行政区', 'HK'],
        ['香港', 'HK'],
        ['MACAO', 'MO'],
        ['MACAU', 'MO'],
        ['澳门特别行政区', 'MO'],
        ['澳门', 'MO'],
        ['TAIWAN', 'TW'],
        ['台湾省', 'TW'],
        ['台湾', 'TW'],
        ['JAPAN', 'JP'],
        ['日本', 'JP'],
        ['KOREA', 'KR'],
        ['韩国', 'KR'],
        ['SINGAPORE', 'SG'],
        ['新加坡', 'SG'],
        ['MALAYSIA', 'MY'],
        ['马来西亚', 'MY'],
        ['THAILAND', 'TH'],
        ['泰国', 'TH'],
        ['VIETNAM', 'VN'],
        ['越南', 'VN'],
        ['PHILIPPINES', 'PH'],
        ['菲律宾', 'PH'],
        ['INDONESIA', 'ID'],
        ['印尼', 'ID'],
        ['UNITEDSTATES', 'US'],
        ['UNITED STATES', 'US'],
        ['USA', 'US'],
        ['美国', 'US'],
        ['CANADA', 'CA'],
        ['加拿大', 'CA'],
        ['BRAZIL', 'BR'],
        ['巴西', 'BR'],
        ['ARGENTINA', 'AR'],
        ['阿根廷', 'AR'],
        ['UNITEDKINGDOM', 'GB'],
        ['UNITED KINGDOM', 'GB'],
        ['BRITAIN', 'GB'],
        ['ENGLAND', 'GB'],
        ['英国', 'GB'],
        ['IRELAND', 'IE'],
        ['爱尔兰', 'IE'],
        ['PORTUGAL', 'PT'],
        ['葡萄牙', 'PT'],
        ['SPAIN', 'ES'],
        ['西班牙', 'ES'],
        ['FRANCE', 'FR'],
        ['法国', 'FR'],
        ['GERMANY', 'DE'],
        ['德国', 'DE'],
        ['NETHERLANDS', 'NL'],
        ['荷兰', 'NL'],
        ['ITALY', 'IT'],
        ['意大利', 'IT'],
        ['RUSSIA', 'RU'],
        ['俄罗斯', 'RU'],
        ['TURKEY', 'TR'],
        ['土耳其', 'TR'],
        ['EGYPT', 'EG'],
        ['埃及', 'EG'],
        ['SOUTHAFRICA', 'ZA'],
        ['SOUTH AFRICA', 'ZA'],
        ['南非', 'ZA'],
        ['UNITEDARABEMIRATES', 'AE'],
        ['UNITED ARAB EMIRATES', 'AE'],
        ['UAE', 'AE'],
        ['DUBAI', 'AE'],
        ['阿联酋', 'AE'],
        ['SAUDI', 'SA'],
        ['沙特', 'SA'],
        ['INDIA', 'IN'],
        ['印度', 'IN'],
        ['PAKISTAN', 'PK'],
        ['巴基斯坦', 'PK'],
        ['KAZAKHSTAN', 'KZ'],
        ['哈萨克斯坦', 'KZ'],
        ['CHINA', 'CN'],
        ['MAINLAND CHINA', 'CN'],
        ['中国大陆', 'CN'],
        ['中国', 'CN'],
        ['AUSTRALIA', 'AU'],
        ['澳大利亚', 'AU'],
        ['NEWZEALAND', 'NZ'],
        ['NEW ZEALAND', 'NZ'],
        ['新西兰', 'NZ'],
        ['CHILE', 'CL'],
        ['智利', 'CL'],
        ['COLOMBIA', 'CO'],
        ['哥伦比亚', 'CO'],
        ['PERU', 'PE'],
        ['秘鲁', 'PE'],
        ['SWEDEN', 'SE'],
        ['瑞典', 'SE'],
        ['NORWAY', 'NO'],
        ['挪威', 'NO'],
        ['FINLAND', 'FI'],
        ['芬兰', 'FI'],
        ['POLAND', 'PL'],
        ['波兰', 'PL'],
        ['UKRAINE', 'UA'],
        ['乌克兰', 'UA'],
        ['SWITZERLAND', 'CH'],
        ['瑞士', 'CH'],
        ['AUSTRIA', 'AT'],
        ['奥地利', 'AT'],
        ['BELGIUM', 'BE'],
        ['比利时', 'BE'],
        ['DENMARK', 'DK'],
        ['丹麦', 'DK'],
        ['ROMANIA', 'RO'],
        ['罗马尼亚', 'RO'],
        ['HUNGARY', 'HU'],
        ['匈牙利', 'HU']
    ]);

    var MODE_THEMES = Object.freeze({
        public: {
            hubColor: '#22d3ee',
            stableColor: '#22d3ee',
            warnColor: '#60a5fa',
            alertColor: '#f59e0b',
            areaColor: '#10243d',
            borderColor: '#35537a',
            areaHoverColor: '#173255',
            lineOpacity: 0.42
        },
        admin: {
            hubColor: '#38bdf8',
            stableColor: '#22d3ee',
            warnColor: '#a78bfa',
            alertColor: '#fb7185',
            areaColor: '#0f1e36',
            borderColor: '#40618f',
            areaHoverColor: '#19345d',
            lineOpacity: 0.5
        },
        command: {
            hubColor: '#22d3ee',
            stableColor: '#22d3ee',
            warnColor: '#f59e0b',
            alertColor: '#fb7185',
            areaColor: '#11253f',
            borderColor: '#3b82f6',
            areaHoverColor: '#17355e',
            lineOpacity: 0.58
        }
    });

    function normalizeLocation(value) {
        return String(value || '')
            .trim()
            .toUpperCase()
            .replace(/[^A-Z0-9\u4E00-\u9FFF]+/g, ' ')
            .replace(/\s+/g, ' ')
            .trim();
    }

    function compactLocation(value) {
        return normalizeLocation(value).replace(/\s+/g, '');
    }

    function resolveLocationKey() {
        var values = Array.prototype.slice.call(arguments)
            .filter(function (item) {
                return String(item || '').trim() !== '';
            })
            .sort(function (left, right) {
                return String(right || '').trim().length - String(left || '').trim().length;
            });
        for (var index = 0; index < values.length; index += 1) {
            var raw = String(values[index] || '').trim();
            if (!raw) {
                continue;
            }

            var direct = raw.toUpperCase();
            if (COUNTRY_COORDS[direct]) {
                return direct;
            }

            var normalized = normalizeLocation(raw);
            var compact = compactLocation(raw);

            if (COUNTRY_COORDS[normalized]) {
                return normalized;
            }
            if (COUNTRY_COORDS[compact]) {
                return compact;
            }

            var tokenPool = [normalized, compact]
                .concat(normalized.split(/\s+/))
                .concat(direct.split(/[^A-Z0-9]+/))
                .map(function (item) { return String(item || '').trim(); })
                .filter(Boolean);

            for (var tokenIndex = 0; tokenIndex < tokenPool.length; tokenIndex += 1) {
                if (COUNTRY_COORDS[tokenPool[tokenIndex]]) {
                    return tokenPool[tokenIndex];
                }
            }

            for (var aliasIndex = 0; aliasIndex < LOCATION_ALIASES.length; aliasIndex += 1) {
                var alias = LOCATION_ALIASES[aliasIndex];
                var needle = alias[0];
                var compactNeedle = needle.replace(/\s+/g, '');
                if (normalized.indexOf(needle) !== -1 || compact.indexOf(compactNeedle) !== -1) {
                    return alias[1];
                }
            }
        }

        return '';
    }

    function coordFor() {
        var key = resolveLocationKey.apply(null, arguments);
        return key && COUNTRY_COORDS[key] ? COUNTRY_COORDS[key].slice() : null;
    }

    function clamp(value, min, max) {
        return Math.max(min, Math.min(max, value));
    }

    function seriesColor(theme, tone) {
        if (tone === 'alert' || tone === 'rose') {
            return theme.alertColor;
        }
        if (tone === 'warn' || tone === 'amber' || tone === 'violet') {
            return theme.warnColor;
        }
        return theme.stableColor;
    }

    function escapeHtml(value) {
        return String(value == null ? '' : value)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    function buildTooltipHtml(params) {
        var meta = params && params.data && params.data.meta ? params.data.meta : {};
        var title = meta.label || meta.name || params.name || '-';
        var lines = [];

        if (meta.key) {
            lines.push(meta.key);
        }
        if (typeof meta.count !== 'undefined') {
            lines.push('节点/来源 ' + meta.count);
        }
        if (typeof meta.online !== 'undefined') {
            lines.push('在线 ' + meta.online);
        }
        if (meta.detail) {
            lines.push(meta.detail);
        }

        return '<div style="min-width:160px">' +
            '<div style="font-weight:700;margin-bottom:4px;color:#f8fafc">' + escapeHtml(title) + '</div>' +
            lines.map(function (line) {
                return '<div style="font-size:12px;color:#cbd5e1;line-height:1.5">' + escapeHtml(line) + '</div>';
            }).join('') +
            '</div>';
    }

    function ensureChart(container) {
        var chart = window.echarts.getInstanceByDom(container);
        if (!chart) {
            chart = window.echarts.init(container);
            if (window.ResizeObserver) {
                var observer = new window.ResizeObserver(function () {
                    chart.resize();
                });
                observer.observe(container);
                container.__notxboardGeoObserver = observer;
            }
        }
        return chart;
    }

    function render(container, config) {
        if (!container || !window.echarts || !window.echarts.getMap || !window.echarts.getMap('world')) {
            return null;
        }

        var mode = config && config.mode ? config.mode : 'public';
        var theme = MODE_THEMES[mode] || MODE_THEMES.public;
        var chart = ensureChart(container);
        var hub = config && config.hub && Array.isArray(config.hub.coord) ? config.hub : {
            name: '平台调度中心',
            label: '平台调度中心',
            coord: [104.1954, 35.8617]
        };
        var points = Array.isArray(config && config.points) ? config.points : [];
        var links = Array.isArray(config && config.links) ? config.links : [];
        var minSize = Number(config && config.minPointSize || 12);
        var maxSize = Number(config && config.maxPointSize || 28);
        var maxValue = Math.max(1, points.reduce(function (currentMax, point) {
            return Math.max(currentMax, Number(point && point.count || 0));
        }, 1));

        var lineData = links
            .filter(function (link) {
                return Array.isArray(link && link.coords) && link.coords.length === 2;
            })
            .map(function (link) {
                var tone = link.tone || 'stable';
                return {
                    coords: link.coords,
                    value: Number(link.value || 0),
                    meta: {
                        label: link.label || link.name || '链路',
                        detail: link.detail || '',
                        count: Number(link.value || 0)
                    },
                    lineStyle: {
                        color: seriesColor(theme, tone),
                        width: tone === 'alert' || tone === 'rose' ? 2.6 : 1.8,
                        opacity: theme.lineOpacity,
                        curveness: typeof link.curveness === 'number' ? link.curveness : 0.18
                    }
                };
            });

        var maxLabels = Number(config && config.maxLabels || 0);
        var pointData = points
            .filter(function (point) {
                return Array.isArray(point && point.coord) && point.coord.length === 2;
            })
            .map(function (point, index) {
                var tone = point.tone || 'stable';
                var count = Math.max(1, Number(point.count || 1));
                var symbolSize = clamp(minSize + (count / maxValue) * (maxSize - minSize), minSize, maxSize);
                return {
                    name: point.label || point.key || '',
                    value: [point.coord[0], point.coord[1], count],
                    symbolSize: symbolSize,
                    meta: point,
                    label: {
                        show: !!(config && config.showLabels) && (!maxLabels || index < maxLabels),
                        position: 'top',
                        distance: 8,
                        color: '#e2e8f0',
                        fontSize: mode === 'command' ? 11 : 10,
                        formatter: point.shortLabel || point.label || point.key || ''
                    },
                    itemStyle: {
                        color: seriesColor(theme, tone),
                        borderColor: '#f8fafc',
                        borderWidth: tone === 'alert' || tone === 'rose' ? 1.8 : 1.2,
                        shadowBlur: tone === 'alert' || tone === 'rose' ? 26 : 18,
                        shadowColor: seriesColor(theme, tone)
                    }
                };
            });

        var hubData = [{
            name: hub.label || hub.name || '调度中心',
            value: [hub.coord[0], hub.coord[1], 1],
            symbolSize: Number(config && config.hubSize || (mode === 'command' ? 22 : 18)),
            meta: {
                label: hub.label || hub.name || '调度中心',
                detail: hub.detail || '',
                key: 'HUB'
            },
            itemStyle: {
                color: theme.hubColor,
                borderColor: '#f8fafc',
                borderWidth: 1.8,
                shadowBlur: 24,
                shadowColor: theme.hubColor
            },
            label: {
                show: !!(config && config.showHubLabel),
                position: 'bottom',
                distance: 10,
                color: '#f8fafc',
                fontWeight: 600,
                formatter: hub.label || hub.name || '调度中心'
            }
        }];

        var geoOptions = {
            map: 'world',
            roam: false,
            silent: true,
            layoutCenter: ['50%', '50%'],
            layoutSize: mode === 'command' ? '96%' : '94%',
            itemStyle: {
                areaColor: theme.areaColor,
                borderColor: theme.borderColor,
                borderWidth: 0.8
            },
            emphasis: {
                itemStyle: {
                    areaColor: theme.areaHoverColor
                }
            },
            label: {
                show: false
            }
        };

        if (typeof config !== 'undefined' && Array.isArray(config && config.center) && config.center.length === 2) {
            geoOptions.center = config.center;
        }
        if (typeof config !== 'undefined' && typeof (config && config.zoom) === 'number' && Number.isFinite(config.zoom)) {
            geoOptions.zoom = config.zoom;
        }

        chart.setOption({
            backgroundColor: 'transparent',
            animationDuration: 600,
            animationDurationUpdate: 350,
            tooltip: {
                trigger: 'item',
                confine: true,
                backgroundColor: 'rgba(2, 6, 23, 0.94)',
                borderColor: 'rgba(96, 165, 250, 0.4)',
                borderWidth: 1,
                textStyle: {
                    color: '#e2e8f0'
                },
                formatter: buildTooltipHtml
            },
            geo: geoOptions,
            series: [
                {
                    name: '链路',
                    type: 'lines',
                    coordinateSystem: 'geo',
                    zlevel: 1,
                    effect: {
                        show: true,
                        constantSpeed: mode === 'command' ? 36 : 28,
                        trailLength: 0.28,
                        symbol: 'circle',
                        symbolSize: mode === 'command' ? 4.5 : 3.5
                    },
                    lineStyle: {
                        width: 1.4,
                        opacity: theme.lineOpacity,
                        curveness: 0.18
                    },
                    data: lineData
                },
                {
                    name: '节点热点',
                    type: 'effectScatter',
                    coordinateSystem: 'geo',
                    zlevel: 3,
                    rippleEffect: {
                        scale: mode === 'command' ? 4 : 3.2,
                        brushType: 'stroke'
                    },
                    itemStyle: {
                        shadowBlur: 18
                    },
                    data: pointData
                },
                {
                    name: '调度中心',
                    type: 'effectScatter',
                    coordinateSystem: 'geo',
                    zlevel: 4,
                    rippleEffect: {
                        scale: mode === 'command' ? 5 : 4,
                        brushType: 'stroke'
                    },
                    data: hubData
                }
            ]
        }, true);

        chart.resize();
        return chart;
    }

    function dispose(container) {
        var chart = container ? window.echarts.getInstanceByDom(container) : null;
        if (chart) {
            chart.dispose();
        }
        if (container && container.__notxboardGeoObserver) {
            container.__notxboardGeoObserver.disconnect();
            delete container.__notxboardGeoObserver;
        }
    }

    window.NotXboardGeoMap = {
        render: render,
        dispose: dispose,
        resolveLocationKey: resolveLocationKey,
        coordFor: coordFor
    };
}(window));
