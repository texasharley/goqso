#!/usr/bin/env python3
"""
Validate prefix_rules.json against authoritative dxcc_entities.json.
This checks the JSON-to-JSON mappings, not the Rust code.
"""

import json
from pathlib import Path
from collections import defaultdict

def main():
    base = Path(__file__).parent.parent
    entities_path = base / "src-tauri" / "resources" / "dxcc_entities.json"
    rules_path = base / "src-tauri" / "resources" / "prefix_rules.json"
    
    # Load authoritative entity data
    with open(entities_path, 'r') as f:
        entities = json.load(f)
    
    # Build entity lookups
    entity_by_id = {}
    active_entities = set()
    for e in entities:
        eid = int(e['EntityId'])
        entity_by_id[eid] = {
            'name': e['Name'],
            'deleted': e.get('Deleted', False),
            'prefixes': e.get('Prefixes'),
        }
        if not e.get('Deleted', False):
            active_entities.add(eid)
    
    # Load prefix rules
    with open(rules_path, 'r') as f:
        rules_data = json.load(f)
    
    rules = rules_data['rules']
    
    print(f"=== PREFIX RULES VALIDATION ===")
    print(f"Active DXCC entities: {len(active_entities)}")
    print(f"Total prefix rules: {len(rules)}")
    print()
    
    # Check for errors
    errors = []
    warnings = []
    covered_entities = set()
    
    for rule in rules:
        prefix = rule['prefix']
        entity_id = rule['entity_id']
        comment = rule.get('comment', '')
        
        # Check if entity exists
        if entity_id not in entity_by_id:
            errors.append(f"  {prefix} -> {entity_id}: Entity ID does not exist")
            continue
        
        entity = entity_by_id[entity_id]
        
        # Check if entity is deleted
        if entity['deleted']:
            errors.append(f"  {prefix} -> {entity_id} ({entity['name']}): Entity is DELETED")
            continue
        
        # Check if comment matches entity name
        if comment and comment != entity['name']:
            # Just a warning - names can have variations
            if entity['name'].lower() not in comment.lower() and comment.lower() not in entity['name'].lower():
                warnings.append(f"  {prefix}: Comment '{comment}' doesn't match entity '{entity['name']}'")
        
        covered_entities.add(entity_id)
    
    # Check coverage
    missing_entities = active_entities - covered_entities
    
    print(f"=== COVERAGE ===")
    print(f"Entities with rules: {len(covered_entities)}/{len(active_entities)}")
    print(f"Coverage: {100 * len(covered_entities) / len(active_entities):.1f}%")
    print()
    
    if missing_entities:
        print(f"=== MISSING ENTITIES ({len(missing_entities)}) ===")
        for eid in sorted(missing_entities):
            e = entity_by_id[eid]
            print(f"  {eid}: {e['name']} (prefixes: {e['prefixes']})")
        print()
    
    if errors:
        print(f"=== ERRORS ({len(errors)}) ===")
        for err in errors:
            print(err)
        print()
    
    if warnings:
        print(f"=== WARNINGS ({len(warnings)}) ===")
        for w in warnings[:20]:
            print(w)
        if len(warnings) > 20:
            print(f"  ... and {len(warnings) - 20} more")
        print()
    
    # Check for duplicate prefix->entity pairs
    seen = defaultdict(list)
    for rule in rules:
        key = rule['prefix']
        seen[key].append(rule['entity_id'])
    
    duplicates = {k: v for k, v in seen.items() if len(v) != len(set(v))}
    if duplicates:
        print(f"=== DUPLICATE RULES ({len(duplicates)}) ===")
        for prefix, eids in list(duplicates.items())[:10]:
            print(f"  {prefix}: {eids}")
        print()
    
    # Final summary
    print(f"=== SUMMARY ===")
    print(f"Total rules: {len(rules)}")
    print(f"Unique prefixes: {len(seen)}")
    print(f"Coverage: {len(covered_entities)}/{len(active_entities)} entities ({100 * len(covered_entities) / len(active_entities):.1f}%)")
    print(f"Errors: {len(errors)}")
    print(f"Warnings: {len(warnings)}")
    
    if len(errors) == 0 and len(covered_entities) == len(active_entities):
        print("\n✅ VALIDATION PASSED: Full coverage, no errors")
        return 0
    elif len(errors) == 0:
        print(f"\n⚠️  VALIDATION WARNING: {len(missing_entities)} entities missing")
        return 0
    else:
        print(f"\n❌ VALIDATION FAILED: {len(errors)} errors")
        return 1

if __name__ == "__main__":
    exit(main())
