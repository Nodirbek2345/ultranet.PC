import re

with open('lib/main.dart', 'r', encoding='utf-8') as f:
    code = f.read()

replaces = [
    ("const Text('Ping Optimizer'", "Text(AppLocales.get('ping_optimizer')"),
    ("'DNS Flush, TCP Analysis, MTU, Power Plan va boshqa optimallashtirish qadamlari.'", "AppLocales.get('ai_subtitle')"), # Oops, bad string, I will just do exact replaces.
    ("Text(_running ? 'Optimallashtirish...' : 'Start Optimization')", "Text(_running ? AppLocales.get('optimizing') : AppLocales.get('start_optimization'))"),
    ("const Text('Before / After'", "Text('${AppLocales.get('before_optimization')} / ${AppLocales.get('after_optimization')}'"),
    ("_CompareRow('Ping'", "_CompareRow('Ping'"),
    ("_CompareRow('Jitter'", "_CompareRow(AppLocales.get('jitter')"),
    ("_CompareRow('Loss'", "_CompareRow(AppLocales.get('loss')"),
    ("const Text('Bajarilgan qadamlar'", "Text(AppLocales.get('optimization_steps')"),
    
    ("const Text('AI Diagnostics'", "Text(AppLocales.get('ai_diagnostics')"),
    ("Text('Lokal AI — har bir tarmoq muammosini avtomatik aniqlab, sabab va yechimini ko\\'rsatadi.'", "Text(AppLocales.get('ai_subtitle')"),
    ("const Text('Qayta tekshirish')", "Text(AppLocales.get('start_optimization'))"), # I'll just use raw replace
    
    ("_DetailRow('Sabab'", "_DetailRow('Sabab'"), # I'll do this in python manually.
]

# Just a simple python script to do it. Wait, I will use multi_replace instead.
