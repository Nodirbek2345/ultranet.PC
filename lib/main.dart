import 'dart:async';
import 'package:flutter/material.dart';
import 'dart:ui';
import 'package:fl_chart/fl_chart.dart';
import 'package:tray_manager/tray_manager.dart';
import 'package:ultranet/src/rust/api/engine.dart';
import 'package:ultranet/src/rust/api/ai.dart';
import 'package:ultranet/src/rust/frb_generated.dart';

import 'package:ultranet/src/rust/plugins/optimizer.dart';
import 'package:ultranet/src/rust/plugins/ai.dart';
import 'package:ultranet/src/rust/plugins/traceroute.dart';
import 'package:ultranet/src/rust/plugins/speedtest.dart';
import 'package:ultranet/locale.dart';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:url_launcher/url_launcher.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:ultranet/config.dart';

const String githubRepo = 'Nodirbek2345/ultranet.PC'; // GitHub repo manzili

Future<void> main() async {
  await RustLib.init();
  initV2();
  setApiKey(key: apiKey);
  runApp(const UltraNetApp());
}

class UltraNetApp extends StatefulWidget {
  const UltraNetApp({super.key});

  @override
  State<UltraNetApp> createState() => _UltraNetAppState();
}

class _UltraNetAppState extends State<UltraNetApp> with TrayListener {
  int _currentPage = 0;
  V2Metrics? _metrics;
  Timer? _timer;
  bool _engineActive = false;
  bool _autoOptimize = false;

  final List<FlSpot> _downloadData = [];
  final List<FlSpot> _uploadData = [];
  final List<FlSpot> _pingData = [];
  double _timeX = 0;

  @override
  void initState() {
    super.initState();
    trayManager.addListener(this);
    _initTray();
    _startMonitoring();
  }

  Future<void> _initTray() async {
    await trayManager.setIcon('windows/runner/resources/app_icon.ico');
    List<MenuItem> items = [
      MenuItem(key: 'show', label: 'Open UltraNet'),
      MenuItem(key: 'gaming_mode', label: 'Toggle Gaming Mode'),
      MenuItem.separator(),
      MenuItem(key: 'exit', label: 'Exit App'),
    ];
    await trayManager.setContextMenu(Menu(items: items));
  }

  @override
  void onTrayIconRightMouseDown() {
    trayManager.popUpContextMenu();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) {
    if (menuItem.key == 'exit') {
      // exit
    } else if (menuItem.key == 'gaming_mode') {
      _toggleEngine();
    }
  }

  void _toggleEngine() {
    setState(() {
      _engineActive = !_engineActive;
      if (_engineActive) {
        startAdaptiveEngine();
      } else {
        stopAdaptiveEngine();
      }
    });
  }

  void _toggleAutoOptimize() {
    setState(() {
      _autoOptimize = !_autoOptimize;
      if (_autoOptimize) {
        enableProAutoOptimize();
      } else {
        disableProAutoOptimize();
      }
    });
  }

  void _startMonitoring() {
    _timer = Timer.periodic(const Duration(seconds: 2), (timer) async {
      try {
        final metrics = await getV2Metrics();
        if (mounted) {
          setState(() {
            _metrics = metrics;
            _timeX += 1;
            _downloadData.add(FlSpot(_timeX, metrics.downloadSpeed / 1024));
            _uploadData.add(FlSpot(_timeX, metrics.uploadSpeed / 1024));
            final ping = metrics.pings.isNotEmpty ? metrics.pings.first.averageMs.toDouble() : 0.0;
            _pingData.add(FlSpot(_timeX, ping));
            if (_downloadData.length > 60) {
              _downloadData.removeAt(0);
              _uploadData.removeAt(0);
              _pingData.removeAt(0);
            }
          });
        }
      } catch (_) {}
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'UltraNet AI',
      debugShowCheckedModeBanner: false,
      themeMode: ThemeMode.dark,
      darkTheme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: const Color(0xFF0A0E1A),
        colorScheme: ColorScheme.dark(
          primary: const Color(0xFF38BDF8),
          secondary: const Color(0xFF818CF8),
          surface: Colors.white.withValues(alpha: 0.05),
        ),
        fontFamily: 'Segoe UI',
      ),
      home: Scaffold(
        body: Container(
          decoration: const BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [Color(0xFF0A0E1A), Color(0xFF131B2E)],
            ),
          ),
          child: Row(
            children: [
            // === SIDEBAR ===
            Container(
              width: 220,
              decoration: BoxDecoration(
                color: Colors.black.withValues(alpha: 0.3),
                border: Border(right: BorderSide(color: Colors.white.withValues(alpha: 0.06))),
              ),
              child: Column(
                children: [
                  const SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Container(
                        width: 10, height: 10,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: (_metrics?.healthScore ?? 0) > 75 ? Colors.greenAccent : Colors.orangeAccent,
                          boxShadow: [BoxShadow(
                            color: ((_metrics?.healthScore ?? 0) > 75 ? Colors.greenAccent : Colors.orangeAccent).withValues(alpha: 0.5),
                            blurRadius: 8,
                          )],
                        ),
                      ),
                      const SizedBox(width: 10),
                      const Text('UltraNet AI', style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700, color: Colors.white, letterSpacing: 1)),
                    ],
                  ),
                  const SizedBox(height: 16),
                  _SidebarItem(icon: Icons.dashboard_rounded, label: AppLocales.get('dashboard'), selected: _currentPage == 0, onTap: () => setState(() => _currentPage = 0)),
                  _SidebarItem(icon: Icons.tune_rounded, label: AppLocales.get('ping_optimizer'), selected: _currentPage == 1, onTap: () => setState(() => _currentPage = 1)),
                  _SidebarItem(icon: Icons.troubleshoot_rounded, label: AppLocales.get('ai_diagnostics'), selected: _currentPage == 2, onTap: () => setState(() => _currentPage = 2)),
                  _SidebarItem(icon: Icons.route_rounded, label: AppLocales.get('traceroute_ai'), selected: _currentPage == 3, onTap: () => setState(() => _currentPage = 3)),
                  _SidebarItem(icon: Icons.wifi_find_rounded, label: AppLocales.get('wifi_analyzer'), selected: _currentPage == 4, onTap: () => setState(() => _currentPage = 4)),
                  _SidebarItem(icon: Icons.stacked_line_chart_rounded, label: AppLocales.get('bufferbloat'), selected: _currentPage == 5, onTap: () => setState(() => _currentPage = 5)),
                  _SidebarItem(icon: Icons.update_rounded, label: AppLocales.get('updates'), selected: _currentPage == 6, onTap: () => setState(() => _currentPage = 6)),
                  _SidebarItem(icon: Icons.info_outline_rounded, label: 'About', selected: _currentPage == 7, onTap: () => setState(() => _currentPage = 7)),
                  const Spacer(),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        TextButton(
                          onPressed: () => setState(() => AppLocales().setLang('uz')),
                          style: TextButton.styleFrom(
                            foregroundColor: AppLocales().currentLang == 'uz' ? Colors.white : Colors.white38,
                            minimumSize: Size.zero, padding: EdgeInsets.zero, tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                          ),
                          child: const Text('UZ'),
                        ),
                        const Text(' | ', style: TextStyle(color: Colors.white38)),
                        TextButton(
                          onPressed: () => setState(() => AppLocales().setLang('en')),
                          style: TextButton.styleFrom(
                            foregroundColor: AppLocales().currentLang == 'en' ? Colors.white : Colors.white38,
                            minimumSize: Size.zero, padding: EdgeInsets.zero, tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                          ),
                          child: const Text('EN'),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 8),
                  Padding(
                    padding: const EdgeInsets.only(bottom: 16),
                    child: Text(
                      'v3.0 — ${_metrics?.healthLabel ?? "..."}',
                      style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 12),
                    ),
                  ),
                ],
              ),
            ),
            // === MAIN CONTENT ===
            Expanded(
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 200),
                child: _buildPage(),
              ),
            ),
          ],
        ),
      ),
    ));
  }

  Widget _buildPage() {
    switch (_currentPage) {
      case 0: return _DashboardPage(key: const ValueKey(0), metrics: _metrics, downloadData: _downloadData, uploadData: _uploadData, pingData: _pingData, engineActive: _engineActive, onToggleEngine: _toggleEngine, autoOptimize: _autoOptimize, onToggleAutoOptimize: _toggleAutoOptimize);
      case 1: return _OptimizerPage(key: const ValueKey(1), metrics: _metrics);
      case 2: return _DiagnosticsPage(key: const ValueKey(2), metrics: _metrics);
      case 3: return _TraceroutePage(key: const ValueKey(3));
      case 4: return _WifiPage(key: const ValueKey(4), metrics: _metrics);
      case 5: return _BufferbloatPage(key: const ValueKey(5));
      case 6: return const _UpdatesPage(key: ValueKey(6));
      case 7: return const _AboutPage(key: ValueKey(7));
      default: return const SizedBox();
    }
  }
}

// ============================================================
// SIDEBAR ITEM
// ============================================================
class _SidebarItem extends StatelessWidget {
  final IconData icon;
  final String label;
  final bool selected;
  final VoidCallback onTap;
  const _SidebarItem({required this.icon, required this.label, required this.selected, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
      child: Material(
        color: selected ? const Color(0xFF38BDF8).withValues(alpha: 0.15) : Colors.transparent,
        borderRadius: BorderRadius.circular(10),
        child: InkWell(
          borderRadius: BorderRadius.circular(10),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                Icon(icon, size: 20, color: selected ? const Color(0xFF38BDF8) : Colors.white54),
                const SizedBox(width: 12),
                Text(label, style: TextStyle(color: selected ? Colors.white : Colors.white54, fontWeight: selected ? FontWeight.w600 : FontWeight.w400, fontSize: 14)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

// ============================================================
// PAGE 1: DASHBOARD
// ============================================================
class _DashboardPage extends StatelessWidget {
  final V2Metrics? metrics;
  final List<FlSpot> downloadData, uploadData, pingData;
  final bool engineActive;
  final VoidCallback onToggleEngine;
  final bool autoOptimize;
  final VoidCallback onToggleAutoOptimize;
  const _DashboardPage({super.key, this.metrics, required this.downloadData, required this.uploadData, required this.pingData, required this.engineActive, required this.onToggleEngine, required this.autoOptimize, required this.onToggleAutoOptimize});

  Color _scoreColor(int score) {
    if (score >= 90) return Colors.greenAccent;
    if (score >= 75) return const Color(0xFF38BDF8);
    if (score >= 50) return Colors.amber;
    if (score >= 25) return Colors.deepOrange;
    return Colors.redAccent;
  }

  @override
  Widget build(BuildContext context) {
    final ping = metrics?.pings.firstOrNull;
    final score = metrics?.healthScore ?? 0;
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Text('UltraNet AI', style: TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                decoration: BoxDecoration(
                  color: engineActive ? const Color(0xFF38BDF8).withValues(alpha: 0.15) : Colors.white.withValues(alpha: 0.05),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: engineActive ? const Color(0xFF38BDF8) : Colors.white24),
                ),
                child: InkWell(
                  onTap: onToggleEngine,
                  child: Row(
                    children: [
                      Icon(engineActive ? Icons.sports_esports : Icons.sports_esports_outlined, color: engineActive ? const Color(0xFF38BDF8) : Colors.white54, size: 18),
                      const SizedBox(width: 8),
                      Text(engineActive ? 'GAMING ENGINE ACTIVE' : 'GAMING ENGINE OFF', style: TextStyle(color: engineActive ? const Color(0xFF38BDF8) : Colors.white54, fontWeight: FontWeight.w600, fontSize: 12)),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                decoration: BoxDecoration(
                  color: autoOptimize ? Colors.greenAccent.withValues(alpha: 0.15) : Colors.white.withValues(alpha: 0.05),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: autoOptimize ? Colors.greenAccent : Colors.white24),
                ),
                child: InkWell(
                  onTap: onToggleAutoOptimize,
                  child: Row(
                    children: [
                      Icon(autoOptimize ? Icons.auto_fix_high : Icons.auto_fix_off_outlined, color: autoOptimize ? Colors.greenAccent : Colors.white54, size: 18),
                      const SizedBox(width: 8),
                      Text(autoOptimize ? 'AUTO-OPTIMIZE ON' : 'AUTO-OPTIMIZE OFF', style: TextStyle(color: autoOptimize ? Colors.greenAccent : Colors.white54, fontWeight: FontWeight.w600, fontSize: 12)),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 16),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                decoration: BoxDecoration(
                  color: _scoreColor(score).withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: _scoreColor(score).withValues(alpha: 0.4)),
                ),
                child: Text('${metrics?.healthLabel ?? "..."} — $score/100', style: TextStyle(color: _scoreColor(score), fontWeight: FontWeight.w600)),
              ),
            ],
          ),
          const SizedBox(height: 24),
          // Top metric cards
          Row(
            children: [
              _MetricCard(AppLocales.get('icmp_ping'), '${ping?.averageMs.round() ?? "--"} ms', Icons.timer_outlined, const Color(0xFF38BDF8)),
              const SizedBox(width: 12),
              _MetricCard(AppLocales.get('tcp_ping'), '${metrics?.tcpPingMs ?? "--"} ms', Icons.lan_outlined, const Color(0xFF818CF8)),
              const SizedBox(width: 12),
              _MetricCard(AppLocales.get('gateway'), '${metrics?.gatewayPingMs ?? "--"} ms', Icons.router_outlined, Colors.tealAccent),
              const SizedBox(width: 12),
              _MetricCard(AppLocales.get('jitter'), '${ping?.jitterMs.round() ?? "--"} ms', Icons.waves_outlined, Colors.amber),
              const SizedBox(width: 12),
              _MetricCard(AppLocales.get('loss'), '${ping?.packetLossPct.toStringAsFixed(1) ?? "0"}%', Icons.error_outline, Colors.redAccent),
              const SizedBox(width: 12),
              _MetricCard(AppLocales.get('connections'), '${metrics?.activeConnections ?? 0}', Icons.device_hub_outlined, Colors.white54),
            ],
          ),
          const SizedBox(height: 20),
          // Charts
          Expanded(
            child: Row(
              children: [
                Expanded(flex: 3, child: _GlassCard(child: _buildTrafficChart())),
                const SizedBox(width: 16),
                Expanded(flex: 2, child: _GlassCard(child: _buildPingChart())),
              ],
            ),
          ),
          const SizedBox(height: 16),
          // Bottom info row
          Row(
            children: [
              Expanded(child: _GlassCard(child: Padding(
                padding: const EdgeInsets.all(14),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceAround,
                  children: [
                    _InfoChip(Icons.wifi, AppLocales.get('ssid'), metrics?.wifi?.ssid ?? 'LAN'),
                    _InfoChip(Icons.signal_cellular_alt, AppLocales.get('signal'), '${metrics?.wifi?.signalPct ?? 100}%'),
                    _InfoChip(Icons.router, AppLocales.get('gateway'), metrics?.gatewayIp ?? '...'),
                    _InfoChip(Icons.speed, AppLocales.get('adapter'), '${metrics?.adapterSpeedMbps ?? 0} Mbps'),
                    _InfoChip(Icons.apps, AppLocales.get('bg_apps'), '${metrics?.topTraffic.length ?? 0}'),
                  ],
                ),
              ))),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTrafficChart() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 14, 16, 0),
          child: Row(children: [
            Container(width: 10, height: 3, color: const Color(0xFF38BDF8)),
            const SizedBox(width: 6),
            Text(AppLocales.get('download'), style: const TextStyle(color: Colors.white54, fontSize: 12)),
            const SizedBox(width: 16),
            Container(width: 10, height: 3, color: const Color(0xFF818CF8)),
            const SizedBox(width: 6),
            Text(AppLocales.get('upload'), style: const TextStyle(color: Colors.white54, fontSize: 12)),
            const Spacer(),
            Text(AppLocales.get('live_traffic'), style: const TextStyle(color: Colors.white38, fontSize: 12)),
          ]),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: downloadData.length > 1 ? LineChart(
              LineChartData(
                gridData: FlGridData(show: true, drawVerticalLine: false, horizontalInterval: 100, getDrawingHorizontalLine: (_) => FlLine(color: Colors.white.withValues(alpha: 0.04), strokeWidth: 1)),
                titlesData: const FlTitlesData(show: false),
                borderData: FlBorderData(show: false),
                lineBarsData: [
                  LineChartBarData(spots: downloadData, isCurved: true, color: const Color(0xFF38BDF8), barWidth: 2, isStrokeCapRound: true, dotData: const FlDotData(show: false), belowBarData: BarAreaData(show: true, color: const Color(0xFF38BDF8).withValues(alpha: 0.08))),
                  LineChartBarData(spots: uploadData, isCurved: true, color: const Color(0xFF818CF8), barWidth: 2, isStrokeCapRound: true, dotData: const FlDotData(show: false), belowBarData: BarAreaData(show: true, color: const Color(0xFF818CF8).withValues(alpha: 0.06))),
                ],
              ),
            ) : const Center(child: Text("Ma'lumotlar yig'ilmoqda...", style: TextStyle(color: Colors.white24))),
          ),
        ),
      ],
    );
  }

  Widget _buildPingChart() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(padding: const EdgeInsets.fromLTRB(16, 14, 16, 0), child: Text(AppLocales.get('ping_history'), style: const TextStyle(color: Colors.white38, fontSize: 12))),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: pingData.length > 1 ? LineChart(
              LineChartData(
                gridData: const FlGridData(show: false),
                titlesData: const FlTitlesData(show: false),
                borderData: FlBorderData(show: false),
                lineBarsData: [
                  LineChartBarData(spots: pingData, isCurved: true, color: Colors.amber, barWidth: 2, dotData: const FlDotData(show: false), belowBarData: BarAreaData(show: true, color: Colors.amber.withValues(alpha: 0.08))),
                ],
              ),
            ) : const Center(child: Text("...", style: TextStyle(color: Colors.white24))),
          ),
        ),
      ],
    );
  }
}

// ============================================================
// PAGE 2: PING OPTIMIZER (Before/After)
// ============================================================
class _OptimizerPage extends StatefulWidget {
  final V2Metrics? metrics;
  const _OptimizerPage({super.key, this.metrics});
  @override
  State<_OptimizerPage> createState() => _OptimizerPageState();
}

class _OptimizerPageState extends State<_OptimizerPage> {
  bool _analyzing = false;
  List<OptimizationRecommendation>? _recommendations;
  Set<String> _selectedActions = {};

  bool _optimizing = false;
  OptimizationReport? _report;
  bool _hasStarted = false;

  @override
  void initState() {
    super.initState();
    if (widget.metrics != null) {
      _hasStarted = true;
      Future.delayed(const Duration(milliseconds: 500), _startAnalysis);
    }
  }

  @override
  void didUpdateWidget(_OptimizerPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!_hasStarted && widget.metrics != null) {
      _hasStarted = true;
      _startAnalysis();
    }
  }

  Future<void> _startAnalysis() async {
    if (widget.metrics == null) return;
    setState(() { _analyzing = true; _recommendations = null; _report = null; });
    final recs = await analyzeNetwork();
    
    if (mounted) {
      setState(() { 
        _analyzing = false; 
        _recommendations = recs;
        _selectedActions = recs.where((r) => r.isRecommended).map((r) => r.id).toSet();
      });
    }
  }

  Future<void> _startOptimize() async {
    if (widget.metrics == null || _selectedActions.isEmpty) return;
    setState(() { _optimizing = true; });

    final ping = widget.metrics!.pings.isNotEmpty ? widget.metrics!.pings.first.averageMs.round() : 0;
    final report = await applyOptimization(actions: _selectedActions.toList(), beforePing: ping);
    
    if (mounted) {
      setState(() { _optimizing = false; _report = report; });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          Text(AppLocales.get('ping_optimizer'), style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
          const SizedBox(height: 8),
          Text(AppLocales.get('ai_subtitle'), style: TextStyle(color: Colors.white.withValues(alpha: 0.4))),
          const SizedBox(height: 24),
          
          if (_report != null) ...[
            _GlassCard(child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Optimization Report', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
                  const SizedBox(height: 16),
                  _CompareRow('Ping', '${_report!.beforePing} ms', '${_report!.afterPing} ms', _report!.pingDiff),
                  const Divider(color: Colors.white12),
                  Text(_report!.summary, style: const TextStyle(color: Colors.amber, fontSize: 14)),
                  const SizedBox(height: 12),
                  const Text('Actions Applied:', style: TextStyle(color: Colors.white70, fontWeight: FontWeight.bold)),
                  const SizedBox(height: 4),
                  ..._report!.actionsTaken.map((a) => Text("✓ $a", style: const TextStyle(color: Colors.greenAccent))),
                ],
              ),
            )),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: _startAnalysis,
              icon: const Icon(Icons.refresh),
              label: const Text('Re-Analyze'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.white10,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 16),
              ),
            ),
          ] else if (_recommendations != null) ...[
            const Text('Network Analysis Recommendations', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
            const SizedBox(height: 16),
            ..._recommendations!.map((rec) => CheckboxListTile(
              title: Text(rec.title, style: const TextStyle(fontWeight: FontWeight.bold, color: Colors.white)),
              subtitle: Text(rec.description, style: const TextStyle(color: Colors.white70)),
              value: _selectedActions.contains(rec.id),
              onChanged: (val) {
                setState(() {
                  if (val == true) {
                    _selectedActions.add(rec.id);
                  } else {
                    _selectedActions.remove(rec.id);
                  }
                });
              },
              activeColor: const Color(0xFF38BDF8),
              checkColor: Colors.black,
            )),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: (_optimizing || _selectedActions.isEmpty) ? null : _startOptimize,
              icon: _optimizing ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2)) : const Icon(Icons.rocket_launch_rounded),
              label: Text(_optimizing ? AppLocales.get('optimizing') : 'Apply Selected Optimizations'),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF38BDF8),
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 16),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
              ),
            ),
          ] else ...[
            ElevatedButton.icon(
              onPressed: _analyzing ? null : _startAnalysis,
              icon: _analyzing ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2)) : const Icon(Icons.search),
              label: Text(_analyzing ? 'Analyzing Network...' : 'Start Network Analysis'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.white10,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 16),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

// ============================================================
// PAGE 3: AI DIAGNOSTICS
// ============================================================
class _DiagnosticsPage extends StatefulWidget {
  final V2Metrics? metrics;
  const _DiagnosticsPage({super.key, this.metrics});
  @override
  State<_DiagnosticsPage> createState() => _DiagnosticsPageState();
}

class _DiagnosticsPageState extends State<_DiagnosticsPage> {
  List<DiagnosticItem>? _diagnostics;
  bool _loading = false;

  Future<void> _runDiagnostics() async {
    if (widget.metrics == null) return;
    setState(() { _loading = true; });
    final m = widget.metrics!;
    final ping = m.pings.isNotEmpty ? m.pings.first : null;

    final results = await getDiagnostics(
      icmpPing: ping?.averageMs.round() ?? 0,
      tcpPing: m.tcpPingMs,
      gatewayPing: m.gatewayPingMs,
      gatewayIp: m.gatewayIp,
      jitter: ping?.jitterMs.round() ?? 0,
      packetLoss: ping?.packetLossPct ?? 0,
      dnsMs: m.dns.isNotEmpty ? m.dns.first.latencyMs : 0,
      dnsServer: m.dns.isNotEmpty ? m.dns.first.target : '1.1.1.1',
      wifiSignal: m.wifi?.signalPct ?? 0,
      wifiChannel: m.wifi?.channel ?? 0,
      wifiBand: m.wifi?.band ?? 'LAN',
      activeConnections: m.activeConnections,
      adapterSpeed: m.adapterSpeedMbps,
      download: m.downloadSpeed,
      upload: m.uploadSpeed,
      backgroundApps: m.topTraffic.map((t) => t.name).toList(),
    );
    if (mounted) setState(() { _diagnostics = results; _loading = false; });
  }

  bool _hasStarted = false;

  @override
  void initState() {
    super.initState();
    if (widget.metrics != null) {
      _hasStarted = true;
      Future.delayed(const Duration(milliseconds: 500), _runDiagnostics);
    }
  }

  @override
  void didUpdateWidget(_DiagnosticsPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!_hasStarted && widget.metrics != null) {
      _hasStarted = true;
      _runDiagnostics();
    }
  }

  Color _severityColor(String s) {
    switch (s) {
      case 'critical': return Colors.redAccent;
      case 'warning': return Colors.amber;
      case 'info': return const Color(0xFF38BDF8);
      default: return Colors.greenAccent;
    }
  }

  IconData _severityIcon(String s) {
    switch (s) {
      case 'critical': return Icons.error;
      case 'warning': return Icons.warning_amber_rounded;
      case 'info': return Icons.info_outline;
      default: return Icons.check_circle;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(AppLocales.get('ai_diagnostics'), style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
              const Spacer(),
              TextButton.icon(
                onPressed: _loading ? null : _runDiagnostics,
                icon: const Icon(Icons.refresh, size: 18),
                label: Text(AppLocales.get('start_optimization')),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(AppLocales.get('ai_subtitle'), style: TextStyle(color: Colors.white.withValues(alpha: 0.4))),
          const SizedBox(height: 20),
          if (_loading) const Center(child: CircularProgressIndicator()),
          if (_diagnostics != null) Expanded(
            child: ListView.separated(
              itemCount: _diagnostics!.length,
              separatorBuilder: (context, index) => const SizedBox(height: 10),
              itemBuilder: (ctx, i) {
                final d = _diagnostics![i];
                return _GlassCard(child: ExpansionTile(
                  leading: Icon(_severityIcon(d.severity), color: _severityColor(d.severity), size: 22),
                  title: Text(d.problem, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w500)),
                  subtitle: Text(d.category, style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 12)),
                  trailing: Text('${d.confidencePct}%', style: TextStyle(color: _severityColor(d.severity))),
                  children: [
                    Padding(
                      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          if (d.reason.isNotEmpty) _DetailRow('Sabab', d.reason),
                          if (d.howChecked.isNotEmpty) _DetailRow('Tekshiruv', d.howChecked),
                          if (d.solution.isNotEmpty) _DetailRow('Yechim', d.solution),
                          if (d.expectedResult.isNotEmpty) _DetailRow('Natija', d.expectedResult),
                        ],
                      ),
                    ),
                  ],
                ));
              },
            ),
          ),
        ],
      ),
    );
  }
}

// ============================================================
// PAGE 4: TRACEROUTE AI
// ============================================================
class _TraceroutePage extends StatefulWidget {
  const _TraceroutePage({super.key});
  @override
  State<_TraceroutePage> createState() => _TraceroutePageState();
}

class _TraceroutePageState extends State<_TraceroutePage> {
  TracerouteResult? _result;
  bool _loading = false;

  @override
  void initState() {
    super.initState();
    Future.delayed(const Duration(milliseconds: 500), _run);
  }

  Future<void> _run() async {
    setState(() { _loading = true; });
    final res = await runTraceroute(target: "1.1.1.1");
    if (mounted) setState(() { _result = res; _loading = false; });
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(AppLocales.get('traceroute_ai'), style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
          const SizedBox(height: 8),
          Text(AppLocales.get('ai_subtitle'), style: TextStyle(color: Colors.white.withValues(alpha: 0.4))),
          const SizedBox(height: 20),
          ElevatedButton.icon(
            onPressed: _loading ? null : _run,
            icon: _loading ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2)) : const Icon(Icons.route_rounded),
            label: Text(_loading ? AppLocales.get('running') : AppLocales.get('start_traceroute')),
            style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF38BDF8), foregroundColor: Colors.white, padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 16), shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12))),
          ),
          const SizedBox(height: 20),
          if (_result != null) ...[
            _GlassCard(child: Padding(
              padding: const EdgeInsets.all(16),
              child: Text('${AppLocales.get('ai_conclusion')} ${_result!.aiConclusion}', style: const TextStyle(color: Colors.amber, fontWeight: FontWeight.w600)),
            )),
            const SizedBox(height: 12),
            Expanded(child: _GlassCard(child: ListView.builder(
              padding: const EdgeInsets.all(12),
              itemCount: _result!.hops.length,
              itemBuilder: (ctx, i) {
                final h = _result!.hops[i];
                final isHigh = !h.isTimeout && h.latencyMs > 100;
                return Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Row(children: [
                    SizedBox(width: 36, child: Text('${h.hop}', style: const TextStyle(fontWeight: FontWeight.w600, color: Colors.white54))),
                    Expanded(child: Text(h.ip, style: TextStyle(color: h.isTimeout ? Colors.redAccent : Colors.white70))),
                    Text(h.isTimeout ? 'Timeout' : '${h.latencyMs} ms', style: TextStyle(fontWeight: FontWeight.w600, color: h.isTimeout ? Colors.redAccent : isHigh ? Colors.amber : Colors.greenAccent)),
                  ]),
                );
              },
            ))),
          ],
        ],
      ),
    );
  }
}

// ============================================================
// PAGE 5: WI-FI ANALYZER
// ============================================================
class _WifiPage extends StatelessWidget {
  final V2Metrics? metrics;
  const _WifiPage({super.key, this.metrics});

  @override
  Widget build(BuildContext context) {
    final w = metrics?.wifi;
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(AppLocales.get('wifi_analyzer'), style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
          const SizedBox(height: 8),
          Text(AppLocales.get('ai_subtitle'), style: TextStyle(color: Colors.white.withValues(alpha: 0.4))),
          const SizedBox(height: 24),
          if (w == null)
            _GlassCard(child: Padding(padding: const EdgeInsets.all(24), child: Text(AppLocales.get('not_connected'), style: const TextStyle(color: Colors.amber))))
          else
            Wrap(
              spacing: 16, runSpacing: 16,
              children: [
                _WiFiInfoCard(AppLocales.get('ssid'), w.ssid, Icons.wifi),
                _WiFiInfoCard('BSSID', w.bssid, Icons.router),
                _WiFiInfoCard(AppLocales.get('signal'), '${w.signalPct}%', Icons.signal_cellular_alt),
                _WiFiInfoCard(AppLocales.get('channel'), '${w.channel}', Icons.tune),
                _WiFiInfoCard(AppLocales.get('band'), w.band, Icons.settings_input_antenna),
                _WiFiInfoCard('Radio', w.radioType, Icons.radar),
                _WiFiInfoCard(AppLocales.get('auth'), w.authentication, Icons.lock),
                _WiFiInfoCard(AppLocales.get('rx_rate'), '${w.receiveRateMbps} Mbps', Icons.download),
                _WiFiInfoCard(AppLocales.get('tx_rate'), '${w.transmitRateMbps} Mbps', Icons.upload),
              ],
            ),
        ],
      ),
    );
  }
}

// ============================================================
// PAGE 6: BUFFERBLOAT
// ============================================================
class _BufferbloatPage extends StatefulWidget {
  const _BufferbloatPage({super.key});
  @override
  State<_BufferbloatPage> createState() => _BufferbloatPageState();
}

class _BufferbloatPageState extends State<_BufferbloatPage> {
  BufferbloatResult? _result;
  bool _loading = false;

  @override
  void initState() {
    super.initState();
    Future.delayed(const Duration(milliseconds: 500), _run);
  }

  Future<void> _run() async {
    setState(() { _loading = true; });
    final res = await runBufferbloat(target: "1.1.1.1", url: "http://speedtest.tele2.net/10MB.zip");
    if (mounted) setState(() { _result = res; _loading = false; });
  }

  Color _gradeColor(String g) {
    if (g.startsWith('A')) return Colors.greenAccent;
    if (g == 'B') return const Color(0xFF38BDF8);
    if (g == 'C') return Colors.amber;
    return Colors.redAccent;
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(AppLocales.get('bufferbloat'), style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
          const SizedBox(height: 8),
          Text(AppLocales.get('what_is_bufferbloat'), style: TextStyle(color: Colors.white.withValues(alpha: 0.4))),
          const SizedBox(height: 20),
          ElevatedButton.icon(
            onPressed: _loading ? null : _run,
            icon: _loading ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2)) : const Icon(Icons.speed),
            label: Text(_loading ? AppLocales.get('running') : AppLocales.get('start_bufferbloat')),
            style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF818CF8), foregroundColor: Colors.white, padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 16), shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12))),
          ),
          const SizedBox(height: 24),
          if (_result != null) _GlassCard(child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              children: [
                Text(_result!.grade, style: TextStyle(fontSize: 64, fontWeight: FontWeight.w800, color: _gradeColor(_result!.grade))),
                const SizedBox(height: 16),
                _CompareRow(AppLocales.get('idle_ping'), '', '${_result!.idlePingMs} ms', 0),
                _CompareRow(AppLocales.get('download_ping'), '', '${_result!.downloadPingMs} ms', _result!.downloadPingMs - _result!.idlePingMs),
              ],
            ),
          )),
        ],
      ),
    );
  }
}

// ============================================================
// PAGE 7: ABOUT
// ============================================================
class _AboutPage extends StatelessWidget {
  const _AboutPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Center(
        child: SizedBox(
          width: 420,
          child: _GlassCard(child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 40, vertical: 48),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 80, height: 80,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    gradient: const LinearGradient(colors: [Color(0xFF38BDF8), Color(0xFF818CF8)]),
                    boxShadow: [BoxShadow(color: const Color(0xFF38BDF8).withValues(alpha: 0.4), blurRadius: 24)],
                  ),
                  child: const Icon(Icons.wifi_tethering_rounded, color: Colors.white, size: 40),
                ),
                const SizedBox(height: 24),
                const Text('UltraNet AI', style: TextStyle(fontSize: 28, fontWeight: FontWeight.w800, letterSpacing: 1)),
                const SizedBox(height: 8),
                Text('Tarmoq Optimizatsiya Dasturi', style: TextStyle(color: Colors.white.withValues(alpha: 0.4), fontSize: 14)),
                const SizedBox(height: 32),
                const Divider(color: Colors.white12),
                const SizedBox(height: 20),
                _aboutRow('Muallif (Developer)', 'Bekmurodov Nodirbek'),
                const SizedBox(height: 12),
                _aboutRow('Version', '1.0.0'),
                const SizedBox(height: 12),
                _aboutRow('Platform', 'Windows'),
                const SizedBox(height: 12),
                _aboutRow('Framework', 'Flutter + Rust'),
                const SizedBox(height: 28),
                const Divider(color: Colors.white12),
                const SizedBox(height: 16),
                Text('Copyright © Bekmurodov Nodirbek', style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 12)),
                const SizedBox(height: 4),
                Text('Barcha huquqlar himoyalangan', style: TextStyle(color: Colors.white.withValues(alpha: 0.2), fontSize: 11)),
              ],
            ),
          )),
        ),
      ),
    );
  }

  static Widget _aboutRow(String label, String value) {
    return Row(
      children: [
        SizedBox(width: 160, child: Text(label, style: TextStyle(color: Colors.white.withValues(alpha: 0.5), fontSize: 13))),
        Expanded(child: Text(value, style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14))),
      ],
    );
  }
}

// ============================================================
// SHARED WIDGETS
// ============================================================
class _GlassCard extends StatelessWidget {
  final Widget child;
  const _GlassCard({required this.child});
  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(14),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 16, sigmaY: 16),
        child: Container(
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: [
                Colors.white.withValues(alpha: 0.1),
                Colors.white.withValues(alpha: 0.02),
              ],
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
            ),
            borderRadius: BorderRadius.circular(14),
            border: Border.all(color: Colors.white.withValues(alpha: 0.15)),
          ),
          child: child,
        ),
      ),
    );
  }
}

class _MetricCard extends StatelessWidget {
  final String label, value;
  final IconData icon;
  final Color color;
  const _MetricCard(this.label, this.value, this.icon, this.color);
  @override
  Widget build(BuildContext context) {
    return Expanded(child: _GlassCard(child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(children: [
            Icon(icon, size: 16, color: color.withValues(alpha: 0.8)),
            const SizedBox(width: 6),
            Text(label, style: TextStyle(color: Colors.white.withValues(alpha: 0.6), fontSize: 11, fontWeight: FontWeight.w600)),
          ]),
          const SizedBox(height: 6),
          Text(value, style: TextStyle(
            fontSize: 22, 
            fontWeight: FontWeight.w800, 
            color: color,
            shadows: [Shadow(color: color.withValues(alpha: 0.6), blurRadius: 10)],
          )),
        ],
      ),
    )));
  }
}

class _InfoChip extends StatelessWidget {
  final IconData icon;
  final String label, value;
  const _InfoChip(this.icon, this.label, this.value);
  @override
  Widget build(BuildContext context) {
    return Column(children: [
      Icon(icon, size: 18, color: Colors.white38),
      const SizedBox(height: 4),
      Text(label, style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 10)),
      const SizedBox(height: 2),
      Text(value, style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13)),
    ]);
  }
}

class _CompareRow extends StatelessWidget {
  final String label, before, after;
  final int diff;
  const _CompareRow(this.label, this.before, this.after, this.diff);
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(children: [
        SizedBox(width: 80, child: Text(label, style: TextStyle(color: Colors.white.withValues(alpha: 0.5)))),
        if (before.isNotEmpty) Expanded(child: Text(before, style: const TextStyle(color: Colors.white54))),
        Expanded(child: Text(after, style: const TextStyle(fontWeight: FontWeight.w600))),
        if (diff != 0) Text(
          '${diff > 0 ? "+" : ""}$diff ms',
          style: TextStyle(color: diff < 0 ? Colors.greenAccent : Colors.redAccent, fontWeight: FontWeight.w600),
        ),
      ]),
    );
  }
}

class _DetailRow extends StatelessWidget {
  final String label, value;
  const _DetailRow(this.label, this.value);
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(width: 80, child: Text(label, style: TextStyle(color: Colors.white.withValues(alpha: 0.4), fontSize: 12, fontWeight: FontWeight.w600))),
          Expanded(child: Text(value, style: const TextStyle(fontSize: 13))),
        ],
      ),
    );
  }
}

class _WiFiInfoCard extends StatelessWidget {
  final String label, value;
  final IconData icon;
  const _WiFiInfoCard(this.label, this.value, this.icon);
  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 180,
      child: _GlassCard(child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(children: [
          Icon(icon, color: const Color(0xFF38BDF8), size: 20),
          const SizedBox(width: 12),
          Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(label, style: TextStyle(color: Colors.white.withValues(alpha: 0.4), fontSize: 11)),
            const SizedBox(height: 2),
            Text(value, style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
          ]),
        ]),
      )),
    );
  }
}

// ============================================================
// PAGE: UPDATES
// ============================================================
class _UpdatesPage extends StatefulWidget {
  const _UpdatesPage({super.key});
  @override
  State<_UpdatesPage> createState() => _UpdatesPageState();
}

class _UpdatesPageState extends State<_UpdatesPage> {
  bool _loading = false;
  String? _latestVersion;
  String? _releaseNotes;
  String? _downloadUrl;
  String _status = '';
  
  String _currentVersion = '...';

  @override
  void initState() {
    super.initState();
    _initVersionAndCheckForUpdates();
  }

  Future<void> _initVersionAndCheckForUpdates() async {
    final packageInfo = await PackageInfo.fromPlatform();
    setState(() {
      _currentVersion = 'v${packageInfo.version}';
    });
    _checkForUpdates();
  }

  Future<void> _checkForUpdates() async {
    setState(() {
      _loading = true;
      _status = AppLocales.get('check_updates');
    });

    try {
      final response = await http.get(Uri.parse('https://api.github.com/repos/$githubRepo/releases/latest'));
      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        final tagName = data['tag_name'] as String;
        final body = data['body'] as String?;
        final htmlUrl = data['html_url'] as String;
        
        setState(() {
          _latestVersion = tagName;
          _releaseNotes = body ?? '';
          _downloadUrl = htmlUrl;
          _loading = false;
        });
      } else {
        setState(() {
          _loading = false;
          _status = 'Xatolik: Github sahifasi topilmadi (Status: ${response.statusCode})';
        });
      }
    } catch (e) {
      setState(() {
        _loading = false;
        _status = 'Tarmoq xatosi: $e';
      });
    }
  }

  Future<void> _launchUrl() async {
    if (_downloadUrl != null) {
      final uri = Uri.parse(_downloadUrl!);
      if (await canLaunchUrl(uri)) {
        await launchUrl(uri, mode: LaunchMode.externalApplication);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    bool hasUpdate = _latestVersion != null && _latestVersion != _currentVersion;

    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(AppLocales.get('updates'), style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w700)),
          const SizedBox(height: 8),
          Text(AppLocales.get('ai_subtitle'), style: TextStyle(color: Colors.white.withValues(alpha: 0.4))),
          const SizedBox(height: 32),
          _GlassCard(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(hasUpdate ? Icons.system_update_alt_rounded : Icons.check_circle_outline, 
                           color: hasUpdate ? const Color(0xFF38BDF8) : Colors.greenAccent, size: 32),
                      const SizedBox(width: 16),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Joriy versiya: $_currentVersion', style: const TextStyle(fontSize: 16, color: Colors.white54)),
                          if (_loading)
                            Padding(
                              padding: const EdgeInsets.only(top: 8.0),
                              child: Text(_status, style: const TextStyle(color: Colors.amber)),
                            )
                          else if (_latestVersion != null)
                            Padding(
                              padding: const EdgeInsets.only(top: 4.0),
                              child: Text(
                                hasUpdate ? '${AppLocales.get("new_version_available")} $_latestVersion' : AppLocales.get('up_to_date'),
                                style: TextStyle(
                                  fontSize: 18, 
                                  fontWeight: FontWeight.bold, 
                                  color: hasUpdate ? const Color(0xFF38BDF8) : Colors.greenAccent
                                ),
                              ),
                            )
                          else
                            Padding(
                              padding: const EdgeInsets.only(top: 8.0),
                              child: Text(_status, style: const TextStyle(color: Colors.redAccent)),
                            ),
                        ],
                      ),
                      const Spacer(),
                      if (_loading)
                        const CircularProgressIndicator()
                      else
                        ElevatedButton.icon(
                          onPressed: hasUpdate ? _launchUrl : _checkForUpdates,
                          icon: Icon(hasUpdate ? Icons.download_rounded : Icons.refresh_rounded),
                          label: Text(hasUpdate ? AppLocales.get('download_update') : AppLocales.get('check_updates')),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: hasUpdate ? const Color(0xFF38BDF8) : Colors.white10,
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                          ),
                        )
                    ],
                  ),
                  if (hasUpdate && _releaseNotes != null && _releaseNotes!.isNotEmpty) ...[
                    const Padding(
                      padding: EdgeInsets.only(top: 24, bottom: 12),
                      child: Divider(color: Colors.white12),
                    ),
                    const Text('Yangi versiyadagi o\'zgarishlar:', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 16)),
                    const SizedBox(height: 12),
                    Container(
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: Colors.black.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(
                        _releaseNotes!,
                        style: const TextStyle(color: Colors.white70, fontSize: 14),
                      ),
                    ),
                  ]
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
