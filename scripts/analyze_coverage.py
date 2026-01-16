#!/usr/bin/env python3
"""
Analyze prefix coverage: Are all DXCC entities reachable via prefix lookup?
"""

import json
import re
from pathlib import Path

def main():
    base = Path(__file__).parent.parent
    json_path = base / "src-tauri" / "resources" / "dxcc_entities.json"
    rs_path = base / "src-tauri" / "src" / "reference" / "prefixes.rs"
    
    # Load entities
    with open(json_path, 'r') as f:
        entities = json.load(f)
    
    active_entities = {int(e['EntityId']): e['Name'] for e in entities if not e.get('Deleted', False)}
    
    # Parse prefixes.rs for entity_ids used
    with open(rs_path, 'r') as f:
        rs_content = f.read()
    
    pattern = r'PrefixRule\s*\{\s*prefix:\s*"([^"]+)",\s*entity_id:\s*(\d+)'
    rs_rules = re.findall(pattern, rs_content)
    
    # Which entity_ids are covered by prefix rules?
    covered_ids = set(int(eid) for _, eid in rs_rules)
    
    print("=== DXCC PREFIX COVERAGE ANALYSIS ===\n")
    print(f"Active DXCC entities (ARRL official): {len(active_entities)}")
    print(f"Unique entity_ids in prefixes.rs: {len(covered_ids)}")
    print(f"Total prefix rules: {len(rs_rules)}")
    
    # Entities NOT covered by any prefix rule
    uncovered = set(active_entities.keys()) - covered_ids
    print(f"\n=== ENTITIES WITH NO PREFIX RULE ({len(uncovered)}) ===")
    for eid in sorted(uncovered):
        name = active_entities[eid]
        # Find what JSON says about this entity's prefix
        for e in entities:
            if int(e['EntityId']) == eid:
                prefixes = e.get('Prefixes', 'NONE')
                print(f"  {eid}: {name} - JSON prefixes: {prefixes}")
                break
    
    # Entity_ids in prefixes.rs that DON'T exist in active entities
    invalid_ids = covered_ids - set(active_entities.keys())
    if invalid_ids:
        print(f"\n=== INVALID ENTITY_IDs IN CODE ({len(invalid_ids)}) ===")
        for eid in sorted(invalid_ids):
            # Find what this ID actually is
            for e in entities:
                if int(e['EntityId']) == eid:
                    status = "DELETED" if e.get('Deleted') else "???"
                    print(f"  {eid}: {e['Name']} ({status})")
                    break
            else:
                print(f"  {eid}: NOT IN JSON AT ALL")
    
    # Coverage by continent
    print(f"\n=== COVERAGE BY CONTINENT ===")
    by_continent = {}
    for e in entities:
        if e.get('Deleted'):
            continue
        cont = e.get('Continent', 'Unknown')
        eid = int(e['EntityId'])
        if cont not in by_continent:
            by_continent[cont] = {'total': 0, 'covered': 0}
        by_continent[cont]['total'] += 1
        if eid in covered_ids:
            by_continent[cont]['covered'] += 1
    
    for cont in sorted(by_continent.keys()):
        data = by_continent[cont]
        pct = 100 * data['covered'] / data['total'] if data['total'] > 0 else 0
        print(f"  {cont}: {data['covered']}/{data['total']} ({pct:.0f}%)")
    
    # Summary
    coverage_pct = 100 * len(covered_ids & set(active_entities.keys())) / len(active_entities)
    print(f"\n=== SUMMARY ===")
    print(f"Entity coverage: {len(covered_ids & set(active_entities.keys()))}/{len(active_entities)} ({coverage_pct:.1f}%)")
    print(f"Uncovered entities: {len(uncovered)}")
    print(f"Invalid entity_ids in code: {len(invalid_ids)}")

if __name__ == "__main__":
    main()
