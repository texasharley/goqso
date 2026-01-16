#!/usr/bin/env python3
"""
Generate prefix_rules.json from dxcc_entities.json.
This creates the authoritative source for prefix-to-DXCC mappings.
"""

import json
from pathlib import Path
from datetime import datetime

def main():
    base = Path(__file__).parent.parent
    json_path = base / "src-tauri" / "resources" / "dxcc_entities.json"
    output_path = base / "src-tauri" / "resources" / "prefix_rules.json"
    
    with open(json_path, 'r') as f:
        entities = json.load(f)
    
    rules = []
    ambiguous_prefixes = {}  # Track prefixes used by multiple entities
    
    # First pass: identify all prefixes and which entities use them
    for e in entities:
        entity_id = int(e['EntityId'])
        name = e['Name']
        deleted = e.get('Deleted', False)
        continent = e.get('Continent', '')
        
        if deleted:
            continue  # Skip deleted entities
        
        prefixes = e.get('Prefixes')
        if prefixes is None:
            continue
        if isinstance(prefixes, str):
            prefixes = [prefixes]
        
        for p in prefixes:
            # Handle ranges like EA6-EH6
            if '-' in p and len(p) > 4:
                # Expand range
                parts = p.split('-')
                if len(parts) == 2:
                    start, end = parts
                    # Simple range expansion for common patterns
                    # e.g., EA6-EH6 -> EA6, EB6, EC6, ED6, EE6, EF6, EG6, EH6
                    if len(start) == 3 and len(end) == 3:
                        base_char = start[0]
                        start_mid = start[1]
                        end_mid = end[1]
                        suffix = start[2:]
                        for c in range(ord(start_mid), ord(end_mid) + 1):
                            expanded = f"{base_char}{chr(c)}{suffix}"
                            if expanded not in ambiguous_prefixes:
                                ambiguous_prefixes[expanded] = []
                            ambiguous_prefixes[expanded].append({
                                'entity_id': entity_id,
                                'name': name,
                                'continent': continent
                            })
                continue
            
            if p not in ambiguous_prefixes:
                ambiguous_prefixes[p] = []
            ambiguous_prefixes[p].append({
                'entity_id': entity_id,
                'name': name,
                'continent': continent
            })
    
    # Second pass: create rules
    for prefix, entities_list in sorted(ambiguous_prefixes.items()):
        if len(entities_list) == 1:
            # Unambiguous
            e = entities_list[0]
            rules.append({
                'prefix': prefix,
                'entity_id': e['entity_id'],
                'priority': 10 + len(prefix) * 10,  # Longer prefixes = higher priority
                'exact': False,
                'comment': e['name']
            })
        else:
            # Ambiguous - need disambiguation rules
            # Add a comment-only entry to document the ambiguity
            names = ', '.join([f"{e['entity_id']}={e['name']}" for e in entities_list])
            # Pick one as default (often the "mainland" or most common)
            # For now, pick the one with lowest entity_id as default
            default = min(entities_list, key=lambda x: x['entity_id'])
            rules.append({
                'prefix': prefix,
                'entity_id': default['entity_id'],
                'priority': 10 + len(prefix) * 10,
                'exact': False,
                'comment': f"{default['name']} (AMBIGUOUS: also {names})",
                'ambiguous': True,
                'alternatives': [e['entity_id'] for e in entities_list if e['entity_id'] != default['entity_id']]
            })
    
    # Add ITU block expansions that aren't in JSON but are valid
    # (These will be added manually or from a separate source)
    
    output = {
        'version': '1.0.0',
        'generated': datetime.now().isoformat(),
        'source': 'Generated from dxcc_entities.json (ARRL official)',
        'note': 'Ambiguous prefixes marked - require manual disambiguation rules',
        'stats': {
            'total_rules': len(rules),
            'ambiguous': len([r for r in rules if r.get('ambiguous', False)])
        },
        'rules': rules
    }
    
    with open(output_path, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"Generated {len(rules)} prefix rules")
    print(f"  Ambiguous: {output['stats']['ambiguous']}")
    print(f"  Output: {output_path}")
    
    # Show ambiguous prefixes that need attention
    print("\n=== AMBIGUOUS PREFIXES (need disambiguation rules) ===")
    for r in rules:
        if r.get('ambiguous'):
            print(f"  {r['prefix']}: {r['comment']}")

if __name__ == "__main__":
    main()
