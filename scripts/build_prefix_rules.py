#!/usr/bin/env python3
"""
Build prefix_rules.json from:
1. Current prefixes.rs (extract rules, fix entity IDs using comment names)
2. dxcc_entities.json (add missing entities)

This creates the authoritative source for prefix-to-DXCC mappings.
"""

import json
import re
from pathlib import Path
from datetime import datetime
from typing import Optional

def load_entities(json_path: Path) -> tuple[dict, dict]:
    """Load DXCC entities, return (by_id, by_name) lookups."""
    with open(json_path, 'r') as f:
        entities = json.load(f)
    
    by_id = {}
    by_name = {}
    
    for e in entities:
        entity_id = int(e['EntityId'])
        name = e['Name']
        deleted = e.get('Deleted', False)
        
        by_id[entity_id] = {
            'name': name,
            'deleted': deleted,
            'prefixes': e.get('Prefixes'),
            'continent': e.get('Continent', ''),
            'cq_zones': e.get('CqZones'),
            'itu_zones': e.get('ItuZones'),
        }
        
        # Normalize name for lookup (handle variations)
        normalized = normalize_name(name)
        by_name[normalized] = entity_id
        
        # Add common variations
        add_name_variations(by_name, name, entity_id)
    
    return by_id, by_name

def normalize_name(name: str) -> str:
    """Normalize entity name for matching."""
    return name.lower().strip()

def add_name_variations(by_name: dict, name: str, entity_id: int):
    """Add common name variations for fuzzy matching."""
    variations = [
        name.lower(),
        name.lower().replace('.', ''),
        name.lower().replace(' ', ''),
        name.lower().replace('&', 'and'),
        name.lower().replace(' and ', ' & '),
    ]
    
    # Common abbreviation expansions
    abbrev_map = {
        'i.': 'island',
        'is.': 'islands', 
        'is': 'islands',
        'rep.': 'republic',
        'rep of': 'republic of',
        'fed rep of': 'federal republic of',
        'dem rep': 'democratic republic',
        'st.': 'saint',
        'n.': 'northern',
        's.': 'southern',
        'mt.': 'mount',
    }
    
    for abbrev, full in abbrev_map.items():
        if abbrev in name.lower():
            variations.append(name.lower().replace(abbrev, full))
    
    for v in variations:
        if v not in by_name:
            by_name[v] = entity_id

def find_entity_by_name(name: str, by_name: dict, by_id: dict) -> Optional[int]:
    """Find entity ID by name, with fuzzy matching."""
    if not name:
        return None
    
    # Direct lookup
    normalized = normalize_name(name)
    if normalized in by_name:
        return by_name[normalized]
    
    # Try without punctuation
    cleaned = re.sub(r'[^\w\s]', '', normalized)
    if cleaned in by_name:
        return by_name[cleaned]
    
    # Try partial match
    for stored_name, eid in by_name.items():
        if normalized in stored_name or stored_name in normalized:
            return eid
    
    return None

def parse_prefixes_rs(rs_path: Path) -> list[dict]:
    """Parse PrefixRule entries from prefixes.rs."""
    with open(rs_path, 'r') as f:
        content = f.read()
    
    # Match: PrefixRule { prefix: "XX", entity_id: NNN, exact: bool, priority: NN }, // Comment
    pattern = r'PrefixRule\s*\{\s*prefix:\s*"([^"]+)",\s*entity_id:\s*(\d+),\s*exact:\s*(true|false),\s*priority:\s*(\d+)\s*\},?\s*//\s*(.+?)(?:\n|$)'
    
    rules = []
    for match in re.finditer(pattern, content):
        prefix, entity_id, exact, priority, comment = match.groups()
        rules.append({
            'prefix': prefix,
            'original_entity_id': int(entity_id),
            'exact': exact == 'true',
            'priority': int(priority),
            'comment': comment.strip(),
        })
    
    return rules

def get_prefixes_for_entity(entity_data: dict) -> list[str]:
    """Extract prefixes from entity data, handling ranges."""
    prefixes_raw = entity_data.get('prefixes')
    if prefixes_raw is None:
        return []
    
    if isinstance(prefixes_raw, str):
        prefixes_raw = [prefixes_raw]
    
    result = []
    for p in prefixes_raw:
        # Handle ranges like EA6-EH6
        if '-' in p and len(p) > 4:
            expanded = expand_prefix_range(p)
            result.extend(expanded)
        else:
            result.append(p)
    
    return result

def expand_prefix_range(range_str: str) -> list[str]:
    """Expand prefix range like EA6-EH6 to [EA6, EB6, EC6, ...]."""
    parts = range_str.split('-')
    if len(parts) != 2:
        return [range_str]
    
    start, end = parts
    if len(start) != len(end):
        return [range_str]
    
    # Find the varying character position
    vary_pos = -1
    for i, (s, e) in enumerate(zip(start, end)):
        if s != e:
            vary_pos = i
            break
    
    if vary_pos == -1:
        return [range_str]
    
    result = []
    start_char = start[vary_pos]
    end_char = end[vary_pos]
    
    for c in range(ord(start_char), ord(end_char) + 1):
        expanded = start[:vary_pos] + chr(c) + start[vary_pos+1:]
        result.append(expanded)
    
    return result

def build_prefix_rules(by_id: dict, by_name: dict, existing_rules: list[dict]) -> list[dict]:
    """Build complete prefix rules list."""
    rules = []
    covered_entities = set()
    
    # Process existing rules with corrected entity IDs
    for rule in existing_rules:
        comment = rule['comment']
        original_id = rule['original_entity_id']
        
        # Try to find correct entity ID by name
        correct_id = find_entity_by_name(comment, by_name, by_id)
        
        if correct_id is None:
            # Check if original ID is valid
            if original_id in by_id and not by_id[original_id]['deleted']:
                correct_id = original_id
                print(f"  Using original ID for {rule['prefix']}: {original_id} (couldn't match '{comment}')")
            else:
                print(f"  WARNING: Cannot resolve {rule['prefix']} -> '{comment}' (original: {original_id})")
                continue
        
        # Verify the entity exists and is active
        if correct_id not in by_id:
            print(f"  WARNING: Entity {correct_id} not found for {rule['prefix']}")
            continue
        
        if by_id[correct_id]['deleted']:
            print(f"  WARNING: Entity {correct_id} is deleted for {rule['prefix']}")
            continue
        
        rules.append({
            'prefix': rule['prefix'],
            'entity_id': correct_id,
            'priority': rule['priority'],
            'exact': rule['exact'],
            'comment': by_id[correct_id]['name'],
        })
        covered_entities.add(correct_id)
    
    # Add missing entities
    print(f"\nAdding rules for missing entities...")
    active_entities = {eid for eid, data in by_id.items() if not data['deleted']}
    missing = active_entities - covered_entities
    
    for entity_id in sorted(missing):
        entity_data = by_id[entity_id]
        prefixes = get_prefixes_for_entity(entity_data)
        
        if not prefixes:
            print(f"  No prefix for {entity_id}: {entity_data['name']}")
            continue
        
        for prefix in prefixes:
            # Determine priority based on prefix length
            priority = 10 + len(prefix) * 10
            
            rules.append({
                'prefix': prefix,
                'entity_id': entity_id,
                'priority': priority,
                'exact': False,
                'comment': entity_data['name'],
            })
            print(f"  Added: {prefix} -> {entity_id} ({entity_data['name']})")
        
        covered_entities.add(entity_id)
    
    # Sort by prefix for readability
    rules.sort(key=lambda r: (r['prefix'], -r['priority']))
    
    return rules, covered_entities

def add_disambiguation_rules(rules: list[dict], by_id: dict) -> list[dict]:
    """Add disambiguation suffix rules for ambiguous prefixes."""
    
    # Known disambiguation conventions
    # Format: (suffix_prefix, entity_id, comment)
    disambiguation = [
        # HK0 disambiguation
        ('HK0M', 161, 'Malpelo I.'),  # M suffix for Malpelo
        
        # CE0 disambiguation  
        ('CE0Y', 47, 'Easter I.'),      # Y for Easter (Isla de Pascua)
        ('CE0X', 217, 'San Felix & San Ambrosio'),  # X for San Felix
        ('CE0Z', 125, 'Juan Fernandez Is.'),  # Z for Juan Fernandez
        
        # VK9 disambiguation
        ('VK9X', 35, 'Christmas I.'),
        ('VK9C', 38, 'Cocos (Keeling) Is.'),
        ('VK9M', 171, 'Mellish Reef'),
        ('VK9N', 189, 'Norfolk I.'),
        ('VK9W', 303, 'Willis I.'),
        ('VK9L', 147, 'Lord Howe I.'),
        ('VK9H', 111, 'Heard I.'),
        
        # VP8/VP0 disambiguation (Antarctic/Subantarctic)
        ('VP8F', 141, 'Falkland Is.'),
        ('VP8G', 235, 'South Georgia I.'),
        ('VP8O', 238, 'South Orkney Is.'),
        ('VP8H', 240, 'South Sandwich Is.'),
        ('VP8S', 241, 'South Shetland Is.'),
        
        # VP6 disambiguation
        ('VP6D', 513, 'Ducie I.'),
        ('VP6P', 172, 'Pitcairn I.'),
        
        # 3Y disambiguation
        ('3Y0B', 24, 'Bouvet'),  # B for Bouvet
        ('3Y0P', 199, 'Peter 1 I.'),  # P for Peter I
        
        # 3D2 disambiguation
        ('3D2R', 460, 'Rotuma I.'),  # R for Rotuma
        ('3D2C', 489, 'Conway Reef'),  # C for Conway
        
        # JD1 disambiguation
        ('JD1M', 177, 'Minami Torishima'),  # M for Minami
        ('JD1O', 192, 'Ogasawara'),  # O for Ogasawara
        
        # E5 disambiguation
        ('E51', 191, 'North Cook Is.'),  # North
        ('E52', 234, 'South Cook Is.'),  # South (Rarotonga area)
        
        # KH8 disambiguation
        ('KH8S', 515, 'Swains I.'),  # S for Swains
        
        # FO disambiguation
        ('FO0C', 36, 'Clipperton I.'),  # C for Clipperton
        ('FO0M', 509, 'Marquesas Is.'),  # M for Marquesas
        ('FO0A', 508, 'Austral I.'),  # A for Austral
        
        # FK disambiguation
        ('FK0C', 512, 'Chesterfield Is.'),  # C for Chesterfield
        
        # TX disambiguation
        ('TX0C', 36, 'Clipperton I.'),
        ('TX0M', 509, 'Marquesas Is.'),
        
        # French overseas TO prefixes
        ('TO1', 79, 'Guadeloupe'),
        ('TO2', 84, 'Martinique'),
        ('TO4', 453, 'Reunion I.'),
        ('TO5', 99, 'Glorioso Is.'),
        ('TO7', 169, 'Mayotte'),
    ]
    
    for prefix, entity_id, comment in disambiguation:
        # Check if entity exists
        if entity_id not in by_id:
            print(f"  WARNING: Disambiguation entity {entity_id} not found for {prefix}")
            continue
        
        # Higher priority for longer/more specific prefixes
        priority = 20 + len(prefix) * 10
        
        # Check if rule already exists
        exists = any(r['prefix'] == prefix and r['entity_id'] == entity_id for r in rules)
        if not exists:
            rules.append({
                'prefix': prefix,
                'entity_id': entity_id,
                'priority': priority,
                'exact': False,
                'comment': comment,
            })
            print(f"  Added disambiguation: {prefix} -> {entity_id} ({comment})")
    
    return rules

def add_itu_expansions(rules: list[dict], by_id: dict) -> list[dict]:
    """Add ITU callsign block expansions not in JSON."""
    
    # ITU block expansions
    # These are valid callsign prefixes that aren't explicitly listed in ARRL JSON
    itu_expansions = [
        # United States (291) - ITU blocks: A, K, N, W, AA-AL
        ('AA', 291, 'United States of America'),
        ('AB', 291, 'United States of America'),
        ('AC', 291, 'United States of America'),
        ('AD', 291, 'United States of America'),
        ('AE', 291, 'United States of America'),
        ('AF', 291, 'United States of America'),
        ('AG', 291, 'United States of America'),
        ('AH', 291, 'United States of America'),  # Base - territories use AH0-AH9
        ('AI', 291, 'United States of America'),
        ('AJ', 291, 'United States of America'),
        ('AK', 291, 'United States of America'),
        ('AL', 6, 'Alaska'),  # AL specifically for Alaska
        
        # US territories with numeric suffixes
        ('AH0', 166, 'Mariana Is.'),
        ('AH1', 20, 'Baker & Howland Is.'),
        ('AH2', 103, 'Guam'),
        ('AH3', 123, 'Johnston I.'),
        ('AH4', 174, 'Midway I.'),
        ('AH5', 197, 'Palmyra & Jarvis Is.'),  # Kure
        ('AH6', 110, 'Hawaii'),
        ('AH7', 110, 'Hawaii'),
        ('AH8', 9, 'American Samoa'),
        
        ('KH0', 166, 'Mariana Is.'),
        ('KH1', 20, 'Baker & Howland Is.'),
        ('KH2', 103, 'Guam'),
        ('KH3', 123, 'Johnston I.'),
        ('KH4', 174, 'Midway I.'),
        ('KH5', 197, 'Palmyra & Jarvis Is.'),
        ('KH6', 110, 'Hawaii'),
        ('KH7', 110, 'Hawaii'),
        ('KH8', 9, 'American Samoa'),
        ('KH9', 297, 'Wake I.'),
        
        ('WH6', 110, 'Hawaii'),
        ('NH6', 110, 'Hawaii'),
        
        ('KL', 6, 'Alaska'),
        ('KL7', 6, 'Alaska'),
        ('NL', 6, 'Alaska'),
        ('NL7', 6, 'Alaska'),
        ('WL', 6, 'Alaska'),
        ('WL7', 6, 'Alaska'),
        
        ('KP1', 43, 'Desecheo I.'),
        ('KP2', 285, 'US Virgin Is.'),
        ('KP3', 202, 'Puerto Rico'),
        ('KP4', 202, 'Puerto Rico'),
        ('KP5', 43, 'Desecheo I.'),
        ('NP2', 285, 'US Virgin Is.'),
        ('NP3', 202, 'Puerto Rico'),
        ('NP4', 202, 'Puerto Rico'),
        ('WP2', 285, 'US Virgin Is.'),
        ('WP3', 202, 'Puerto Rico'),
        ('WP4', 202, 'Puerto Rico'),
        
        ('KG4', 105, 'Guantanamo Bay'),
        
        # Canada (1) - ITU: VA-VG, VO, VY
        ('VA', 1, 'Canada'),
        ('VB', 1, 'Canada'),
        ('VC', 1, 'Canada'),
        ('VD', 1, 'Canada'),
        ('VE', 1, 'Canada'),
        ('VF', 1, 'Canada'),
        ('VG', 1, 'Canada'),
        ('VO', 1, 'Canada'),
        ('VY', 1, 'Canada'),
        ('CY0', 252, 'Sable I.'),  # Sable Island
        ('CY9', 252, 'St. Paul I.'),  # St. Paul Island
        
        # Mexico (50) - ITU: XA-XI, 4A-4C, 6D-6J
        ('XA', 50, 'Mexico'),
        ('XB', 50, 'Mexico'),
        ('XC', 50, 'Mexico'),
        ('XD', 50, 'Mexico'),
        ('XE', 50, 'Mexico'),
        ('XF', 50, 'Mexico'),
        ('XG', 50, 'Mexico'),
        ('XH', 50, 'Mexico'),
        ('XI', 50, 'Mexico'),
        ('4A', 50, 'Mexico'),
        ('4B', 50, 'Mexico'),
        ('4C', 50, 'Mexico'),
        ('6D', 50, 'Mexico'),
        ('6E', 50, 'Mexico'),
        ('6F', 50, 'Mexico'),
        ('6G', 50, 'Mexico'),
        ('6H', 50, 'Mexico'),
        ('6I', 50, 'Mexico'),
        ('6J', 50, 'Mexico'),
        ('XF4', 204, 'Revillagigedo'),  # Revillagigedo Islands
        
        # UK expansions
        ('2E', 223, 'England'),
        ('2I', 265, 'Northern Ireland'),
        ('2J', 122, 'Jersey'),
        ('2M', 279, 'Scotland'),
        ('2U', 106, 'Guernsey'),
        ('2W', 294, 'Wales'),
        ('M', 223, 'England'),
        ('G', 223, 'England'),
        ('GD', 114, 'Isle of Man'),
        ('GI', 265, 'Northern Ireland'),
        ('GJ', 122, 'Jersey'),
        ('GM', 279, 'Scotland'),
        ('GU', 106, 'Guernsey'),
        ('GW', 294, 'Wales'),
        
        # Germany (230) - ITU: DA-DR
        ('DA', 230, 'Germany (Federal Rep of)'),
        ('DB', 230, 'Germany (Federal Rep of)'),
        ('DC', 230, 'Germany (Federal Rep of)'),
        ('DD', 230, 'Germany (Federal Rep of)'),
        ('DE', 230, 'Germany (Federal Rep of)'),
        ('DF', 230, 'Germany (Federal Rep of)'),
        ('DG', 230, 'Germany (Federal Rep of)'),
        ('DH', 230, 'Germany (Federal Rep of)'),
        ('DI', 230, 'Germany (Federal Rep of)'),
        ('DJ', 230, 'Germany (Federal Rep of)'),
        ('DK', 230, 'Germany (Federal Rep of)'),
        ('DL', 230, 'Germany (Federal Rep of)'),
        ('DM', 230, 'Germany (Federal Rep of)'),
        ('DN', 230, 'Germany (Federal Rep of)'),
        ('DO', 230, 'Germany (Federal Rep of)'),
        ('DP', 230, 'Germany (Federal Rep of)'),
        ('DQ', 230, 'Germany (Federal Rep of)'),
        ('DR', 230, 'Germany (Federal Rep of)'),
        
        # Japan (339) - ITU: JA-JS, 7J-7N, 8J-8N
        ('JA', 339, 'Japan'),
        ('JB', 339, 'Japan'),
        ('JC', 339, 'Japan'),
        ('JD', 339, 'Japan'),  # Base - JD1 is special
        ('JE', 339, 'Japan'),
        ('JF', 339, 'Japan'),
        ('JG', 339, 'Japan'),
        ('JH', 339, 'Japan'),
        ('JI', 339, 'Japan'),
        ('JJ', 339, 'Japan'),
        ('JK', 339, 'Japan'),
        ('JL', 339, 'Japan'),
        ('JM', 339, 'Japan'),
        ('JN', 339, 'Japan'),
        ('JO', 339, 'Japan'),
        ('JP', 339, 'Japan'),
        ('JQ', 339, 'Japan'),
        ('JR', 339, 'Japan'),
        ('JS', 339, 'Japan'),
        ('7J', 339, 'Japan'),
        ('7K', 339, 'Japan'),
        ('7L', 339, 'Japan'),
        ('7M', 339, 'Japan'),
        ('7N', 339, 'Japan'),
        ('8J', 339, 'Japan'),
        ('8K', 339, 'Japan'),
        ('8L', 339, 'Japan'),
        ('8M', 339, 'Japan'),
        ('8N', 339, 'Japan'),
        
        # Russia - Asiatic (015) and European (054)
        ('UA0', 15, 'Asiatic Russia'),
        ('UA8', 15, 'Asiatic Russia'),
        ('UA9', 15, 'Asiatic Russia'),
        ('UB0', 15, 'Asiatic Russia'),
        ('UB8', 15, 'Asiatic Russia'),
        ('UB9', 15, 'Asiatic Russia'),
        ('UC0', 15, 'Asiatic Russia'),
        ('UC8', 15, 'Asiatic Russia'),
        ('UC9', 15, 'Asiatic Russia'),
        ('UA', 54, 'European Russia'),
        ('UA1', 54, 'European Russia'),
        ('UA2', 54, 'European Russia'),
        ('UA3', 54, 'European Russia'),
        ('UA4', 54, 'European Russia'),
        ('UA6', 54, 'European Russia'),
        ('R', 54, 'European Russia'),  # Base
        ('RA', 54, 'European Russia'),
        ('RV', 54, 'European Russia'),
        ('RW', 54, 'European Russia'),
        ('RX', 54, 'European Russia'),
        ('RZ', 54, 'European Russia'),
        
        # Spain (281) and territories
        ('EA', 281, 'Spain'),
        ('EB', 281, 'Spain'),
        ('EC', 281, 'Spain'),
        ('ED', 281, 'Spain'),
        ('EE', 281, 'Spain'),
        ('EF', 281, 'Spain'),
        ('EG', 281, 'Spain'),
        ('EH', 281, 'Spain'),
        ('EA6', 21, 'Balearic Is.'),
        ('EA8', 29, 'Canary Is.'),
        ('EA9', 32, 'Ceuta & Melilla'),
        
        # Italy (248) - ITU: I
        ('I', 248, 'Italy'),
        ('IK', 248, 'Italy'),
        ('IZ', 248, 'Italy'),
        ('IS0', 225, 'Sardinia'),
        ('IM0', 225, 'Sardinia'),
        
        # France (227) and territories
        ('F', 227, 'France'),
        ('TM', 227, 'France'),
        ('FG', 79, 'Guadeloupe'),
        ('FM', 84, 'Martinique'),
        ('FP', 277, 'St. Pierre & Miquelon'),
        ('FS', 213, 'Saint Martin'),
        ('FJ', 516, 'Saint Barthelemy'),
        ('FY', 63, 'French Guiana'),
        ('FK', 162, 'New Caledonia'),
        ('FO', 175, 'French Polynesia'),
        ('FR', 453, 'Reunion I.'),
        ('FT0W', 41, 'Crozet I.'),
        ('FT0X', 131, 'Kerguelen Is.'),
        ('FT0Z', 10, 'Amsterdam & St. Paul Is.'),
        ('FT0G', 99, 'Glorioso Is.'),
        ('FT0J', 124, 'Juan de Nova, Europa'),
        ('FT0T', 276, 'Tromelin I.'),
        ('FW', 298, 'Wallis & Futuna Is.'),
        
        # Portugal (272)
        ('CT', 272, 'Portugal'),
        ('CQ', 272, 'Portugal'),
        ('CR', 272, 'Portugal'),
        ('CS', 272, 'Portugal'),
        ('CT3', 256, 'Madeira Is.'),
        ('CU', 149, 'Azores'),
        
        # Netherlands (263)
        ('PA', 263, 'Netherlands'),
        ('PB', 263, 'Netherlands'),
        ('PC', 263, 'Netherlands'),
        ('PD', 263, 'Netherlands'),
        ('PE', 263, 'Netherlands'),
        ('PF', 263, 'Netherlands'),
        ('PG', 263, 'Netherlands'),
        ('PH', 263, 'Netherlands'),
        ('PI', 263, 'Netherlands'),
        
        # Netherlands Caribbean
        ('PJ2', 517, 'Curacao'),
        ('PJ4', 520, 'Bonaire'),
        ('PJ5', 519, 'Saba & St. Eustatius'),
        ('PJ6', 519, 'Saba & St. Eustatius'),
        ('PJ7', 518, 'Sint Maarten'),
        
        # Brazil (108)
        ('PP', 108, 'Brazil'),
        ('PQ', 108, 'Brazil'),
        ('PR', 108, 'Brazil'),
        ('PS', 108, 'Brazil'),
        ('PT', 108, 'Brazil'),
        ('PU', 108, 'Brazil'),
        ('PV', 108, 'Brazil'),
        ('PW', 108, 'Brazil'),
        ('PX', 108, 'Brazil'),
        ('PY', 108, 'Brazil'),
        ('ZV', 108, 'Brazil'),
        ('ZW', 108, 'Brazil'),
        ('ZX', 108, 'Brazil'),
        ('ZY', 108, 'Brazil'),
        ('ZZ', 108, 'Brazil'),
        ('PY0F', 56, 'Fernando de Noronha'),
        ('PY0S', 253, 'St. Peter & St. Paul Rocks'),
        ('PY0T', 273, 'Trindade & Martim Vaz Is.'),
        
        # Argentina (100)
        ('LO', 100, 'Argentina'),
        ('LP', 100, 'Argentina'),
        ('LQ', 100, 'Argentina'),
        ('LR', 100, 'Argentina'),
        ('LS', 100, 'Argentina'),
        ('LT', 100, 'Argentina'),
        ('LU', 100, 'Argentina'),
        ('LV', 100, 'Argentina'),
        ('LW', 100, 'Argentina'),
        ('AY', 100, 'Argentina'),
        ('AZ', 100, 'Argentina'),
        ('L2', 100, 'Argentina'),
        ('L3', 100, 'Argentina'),
        ('L4', 100, 'Argentina'),
        ('L5', 100, 'Argentina'),
        ('L6', 100, 'Argentina'),
        ('L7', 100, 'Argentina'),
        ('L8', 100, 'Argentina'),
        ('L9', 100, 'Argentina'),
        
        # Australia (150) base
        ('VK', 150, 'Australia'),
        ('AX', 150, 'Australia'),
        
        # Norway (266) and territories
        ('LA', 266, 'Norway'),
        ('LB', 266, 'Norway'),
        ('LC', 266, 'Norway'),
        ('LD', 266, 'Norway'),
        ('LE', 266, 'Norway'),
        ('LF', 266, 'Norway'),
        ('LG', 266, 'Norway'),
        ('LH', 266, 'Norway'),
        ('LI', 266, 'Norway'),
        ('LJ', 266, 'Norway'),
        ('LK', 266, 'Norway'),
        ('LL', 266, 'Norway'),
        ('LM', 266, 'Norway'),
        ('LN', 266, 'Norway'),
        ('JW', 259, 'Svalbard'),
        ('JX', 118, 'Jan Mayen'),
        
        # Denmark (221) and territories
        ('OU', 221, 'Denmark'),
        ('OV', 221, 'Denmark'),
        ('OW', 221, 'Denmark'),
        ('OX', 237, 'Greenland'),
        ('OY', 222, 'Faroe Is.'),
        ('OZ', 221, 'Denmark'),
        ('5P', 221, 'Denmark'),
        ('5Q', 221, 'Denmark'),
        
        # Finland (224) and Aland
        ('OF', 224, 'Finland'),
        ('OG', 224, 'Finland'),
        ('OH', 224, 'Finland'),
        ('OI', 224, 'Finland'),
        ('OJ', 224, 'Finland'),
        ('OH0', 5, 'Aland Is.'),
        ('OJ0', 167, 'Market Reef'),
        
        # Sweden (284)
        ('SA', 284, 'Sweden'),
        ('SB', 284, 'Sweden'),
        ('SC', 284, 'Sweden'),
        ('SD', 284, 'Sweden'),
        ('SE', 284, 'Sweden'),
        ('SF', 284, 'Sweden'),
        ('SG', 284, 'Sweden'),
        ('SH', 284, 'Sweden'),
        ('SI', 284, 'Sweden'),
        ('SJ', 284, 'Sweden'),
        ('SK', 284, 'Sweden'),
        ('SL', 284, 'Sweden'),
        ('SM', 284, 'Sweden'),
        ('7S', 284, 'Sweden'),
        ('8S', 284, 'Sweden'),
        
        # Poland (269)
        ('SN', 269, 'Poland'),
        ('SO', 269, 'Poland'),
        ('SP', 269, 'Poland'),
        ('SQ', 269, 'Poland'),
        ('SR', 269, 'Poland'),
        ('3Z', 269, 'Poland'),
        ('HF', 269, 'Poland'),
        
        # Greece (236)
        ('SV', 236, 'Greece'),
        ('SW', 236, 'Greece'),
        ('SX', 236, 'Greece'),
        ('SY', 236, 'Greece'),
        ('SZ', 236, 'Greece'),
        ('J4', 236, 'Greece'),
        ('SV5', 45, 'Dodecanese'),
        ('SV9', 40, 'Crete'),
        ('SV/A', 180, 'Mount Athos'),
        
        # Belgium (209)
        ('ON', 209, 'Belgium'),
        ('OO', 209, 'Belgium'),
        ('OP', 209, 'Belgium'),
        ('OQ', 209, 'Belgium'),
        ('OR', 209, 'Belgium'),
        ('OS', 209, 'Belgium'),
        ('OT', 209, 'Belgium'),
        
        # Austria (206)
        ('OE', 206, 'Austria'),
        
        # Switzerland (287)
        ('HB', 287, 'Switzerland'),
        ('HE', 287, 'Switzerland'),
        ('HB0', 251, 'Liechtenstein'),
        
        # Czech Republic (503)
        ('OK', 503, 'Czech Republic'),
        ('OL', 503, 'Czech Republic'),
        
        # Slovakia (504)
        ('OM', 504, 'Slovak Republic'),
        
        # Hungary (239)
        ('HA', 239, 'Hungary'),
        ('HG', 239, 'Hungary'),
        
        # Romania (275)
        ('YO', 275, 'Romania'),
        ('YP', 275, 'Romania'),
        ('YQ', 275, 'Romania'),
        ('YR', 275, 'Romania'),
        
        # Ukraine (288)
        ('UR', 288, 'Ukraine'),
        ('US', 288, 'Ukraine'),
        ('UT', 288, 'Ukraine'),
        ('UU', 288, 'Ukraine'),
        ('UV', 288, 'Ukraine'),
        ('UW', 288, 'Ukraine'),
        ('UX', 288, 'Ukraine'),
        ('UY', 288, 'Ukraine'),
        ('UZ', 288, 'Ukraine'),
        ('EM', 288, 'Ukraine'),
        ('EN', 288, 'Ukraine'),
        ('EO', 288, 'Ukraine'),
        
        # Belarus (27)
        ('EU', 27, 'Belarus (Republic of)'),
        ('EV', 27, 'Belarus (Republic of)'),
        ('EW', 27, 'Belarus (Republic of)'),
        
        # Ireland (245)
        ('EI', 245, 'Ireland'),
        ('EJ', 245, 'Ireland'),
        
        # Iceland (242)
        ('TF', 242, 'Iceland'),
        
        # Additional European
        ('ZA', 7, 'Albania'),
        ('ZB2', 233, 'Gibraltar'),
        ('ZC4', 283, 'UK Sovereign Base Areas on Cyprus'),
        ('C3', 203, 'Andorra'),
        ('3A', 260, 'Monaco'),
        ('T7', 278, 'San Marino'),
        ('HV', 295, 'Vatican'),
        ('1A', 246, 'Sovereign Military Order of Malta'),
        ('9H', 257, 'Malta'),
        ('LX', 254, 'Luxembourg'),
        ('9A', 497, 'Croatia'),
        ('S5', 499, 'Slovenia'),
        ('E7', 501, 'Bosnia-Herzegovina'),
        ('Z3', 502, 'North Macedonia (Republic of)'),
        ('YT', 296, 'Serbia'),
        ('YU', 296, 'Serbia'),
        ('4O', 514, 'Montenegro'),
        ('Z6', 522, 'Republic of Kosovo'),
        ('ER', 179, 'Moldova (Republic of)'),
        ('E4', 510, 'Palestine'),
        
        # Caribbean missing
        ('C6', 60, 'Bahamas (Commonwealth of the)'),
        ('V2', 94, 'Antigua & Barbuda'),
        ('V4', 249, 'St. Kitts & Nevis'),
        ('VP2E', 12, 'Anguilla'),
        ('VP2M', 96, 'Montserrat'),
        ('VP2V', 66, 'British Virgin Is.'),
        ('VP5', 89, 'Turks & Caicos Is.'),
        ('VP9', 64, 'Bermuda'),
        ('ZF', 69, 'Cayman Is.'),
        ('J3', 77, 'Grenada'),
        ('J6', 98, 'St. Lucia'),
        ('J7', 95, 'Dominica'),
        ('J8', 158, 'St. Vincent'),
        ('8P', 62, 'Barbados'),
        ('9Y', 90, 'Trinidad & Tobago'),
        ('9Z', 90, 'Trinidad & Tobago'),
        ('P4', 91, 'Aruba'),
        
        # South America
        ('HK', 116, 'Colombia'),
        ('HK0', 216, 'San Andres & Providencia'),  # Default for HK0
        ('HC', 120, 'Ecuador'),
        ('HC8', 71, 'Galapagos Is.'),
        ('OA', 136, 'Peru'),
        ('CP', 104, 'Bolivia'),
        ('CE', 112, 'Chile'),
        ('CE0', 47, 'Easter I.'),  # Default
        ('CE9', 13, 'Antarctica'),
        ('LU', 100, 'Argentina'),
        ('CX', 144, 'Uruguay'),
        ('ZP', 132, 'Paraguay'),
        ('YV', 148, 'Venezuela'),
        ('YV0', 17, 'Aves I.'),
        
        # Asia
        ('HL', 137, 'Republic of Korea'),
        ('DS', 137, 'Republic of Korea'),
        ('BV', 386, 'Taiwan'),
        ('BV9P', 505, 'Pratas I.'),
        ('BS7', 506, 'Scarborough Reef'),
        ('BY', 318, 'China'),
        ('VR', 321, 'Hong Kong'),
        ('XX9', 152, 'Macao'),
        ('DU', 375, 'Philippines'),
        ('DV', 375, 'Philippines'),
        ('DW', 375, 'Philippines'),
        ('DX', 375, 'Philippines'),
        ('DY', 375, 'Philippines'),
        ('DZ', 375, 'Philippines'),
        ('4D', 375, 'Philippines'),
        ('4E', 375, 'Philippines'),
        ('4F', 375, 'Philippines'),
        ('4G', 375, 'Philippines'),
        ('4H', 375, 'Philippines'),
        ('4I', 375, 'Philippines'),
        ('9M', 299, 'West Malaysia'),
        ('9W', 299, 'West Malaysia'),
        ('9M6', 46, 'East Malaysia'),
        ('9M8', 46, 'East Malaysia'),
        ('YB', 327, 'Indonesia'),
        ('YC', 327, 'Indonesia'),
        ('YD', 327, 'Indonesia'),
        ('YE', 327, 'Indonesia'),
        ('YF', 327, 'Indonesia'),
        ('YG', 327, 'Indonesia'),
        ('YH', 327, 'Indonesia'),
        ('HS', 387, 'Thailand'),
        ('XU', 312, 'Cambodia'),
        ('XV', 293, 'Viet Nam'),
        ('3W', 293, 'Viet Nam'),
        ('XZ', 309, 'Myanmar'),
        ('9V', 381, 'Singapore'),
        ('V8', 345, 'Brunei Darussalam'),
        ('A5', 306, 'Bhutan'),
        ('VU', 324, 'India'),
        ('AP', 372, 'Pakistan (Islamic Rep of)'),
        ('4S', 315, 'Sri Lanka'),
        ('4R', 315, 'Sri Lanka'),
        ('8Q', 159, 'Maldives'),
        ('S2', 305, 'Bangladesh'),
        ('9N', 369, 'Nepal'),
        ('EX', 135, 'Kyrgyzstan'),
        ('EY', 262, 'Tajikistan'),
        ('EZ', 280, 'Turkmenistan'),
        ('UK', 292, 'Uzbekistan'),
        ('UN', 130, 'Kazakhstan'),
        ('JT', 363, 'Mongolia'),
        ('JU', 363, 'Mongolia'),
        ('JV', 363, 'Mongolia'),
        
        # Middle East
        ('TA', 390, 'Republic of Turkiye'),
        ('TB', 390, 'Republic of Turkiye'),
        ('TC', 390, 'Republic of Turkiye'),
        ('YI', 333, 'Iraq'),
        ('HZ', 378, 'Saudi Arabia'),
        ('A4', 370, 'Oman'),
        ('A6', 391, 'United Arab Emirates'),
        ('A7', 376, 'Qatar'),
        ('A9', 304, 'Bahrain'),
        ('9K', 348, 'Kuwait'),
        ('OD', 354, 'Lebanon'),
        ('YK', 384, 'Syrian Arab Republic'),
        ('4X', 336, 'Israel'),
        ('4Z', 336, 'Israel'),
        ('JY', 342, 'Jordan'),
        ('EP', 330, 'Iran'),
        ('EK', 14, 'Armenia'),
        ('4J', 18, 'Azerbaijan'),
        ('4K', 18, 'Azerbaijan'),
        ('4L', 75, 'Georgia'),
        ('5B', 215, 'Cyprus'),
        ('C4', 215, 'Cyprus'),
        ('P3', 215, 'Cyprus'),
        
        # Africa
        ('CN', 446, 'Morocco'),
        ('5C', 446, 'Morocco'),
        ('5D', 446, 'Morocco'),
        ('7X', 400, 'Algeria'),
        ('TS', 474, 'Tunisia'),
        ('3V', 474, 'Tunisia'),
        ('5A', 436, 'Libya'),
        ('SU', 478, 'Egypt'),
        ('ST', 466, 'Sudan'),
        ('E3', 51, 'Eritrea'),
        ('ET', 402, 'Ethiopia'),
        ('9Q', 414, 'Democratic Republic of the Congo'),
        ('9T', 414, 'Democratic Republic of the Congo'),
        ('TL', 408, 'Central African Republic'),
        ('TR', 420, 'Gabon'),
        ('TN', 412, 'Republic of the Congo'),
        ('D2', 401, 'Angola'),
        ('9L', 458, 'Sierra Leone'),
        ('EL', 430, 'Liberia'),
        ('5V', 483, 'Togo'),
        ('TU', 428, 'Cote d\'Ivoire'),
        ('6W', 456, 'Senegal'),
        ('C5', 422, 'The Gambia'),
        ('5T', 444, 'Mauritania'),
        ('5U', 409, 'Niger'),
        ('5X', 434, 'Uganda'),
        ('5Z', 430, 'Kenya'),
        ('5H', 470, 'Tanzania'),
        ('9J', 482, 'Zambia'),
        ('Z2', 452, 'Zimbabwe'),
        ('A2', 402, 'Botswana'),
        ('7P', 432, 'Lesotho'),
        ('3DA', 468, 'Kingdom of Eswatini'),
        ('V5', 464, 'Namibia'),
        ('ZS', 462, 'South Africa'),
        ('ZR', 462, 'South Africa'),
        ('ZT', 462, 'South Africa'),
        ('ZU', 462, 'South Africa'),
        ('3B8', 165, 'Mauritius'),
        ('3B9', 4, 'Agalega & St. Brandon Is.'),
        ('FR', 453, 'Reunion I.'),
        ('FH', 169, 'Mayotte'),
        ('D6', 411, 'Comoros'),
        ('5R', 438, 'Madagascar'),
        ('S7', 379, 'Seychelles'),
        ('VQ9', 33, 'Chagos Is.'),
        ('ZD7', 250, 'St. Helena'),
        ('ZD8', 205, 'Ascension I.'),
        ('ZD9', 274, 'Tristan da Cunha & Gough I.'),
        
        # Oceania/Pacific
        ('ZL', 170, 'New Zealand'),
        ('ZM', 170, 'New Zealand'),
        ('A3', 160, 'Tonga'),
        ('3D2', 176, 'Fiji (Republic of)'),  # Default
        ('5W', 190, 'Samoa'),
        ('T2', 489, 'Tuvalu'),
        ('T3', 301, 'Kiribati'),
        ('V6', 168, 'Micronesia'),
        ('V7', 168, 'Marshall Is.'),
        ('T8', 22, 'Palau'),
        ('KC6', 22, 'Palau'),  # Old prefix
        ('KX6', 168, 'Marshall Is.'),  # Old prefix
        ('YJ', 158, 'Vanuatu'),
        ('H4', 185, 'Solomon Is.'),
        ('P2', 163, 'Papua New Guinea'),
        ('VK9', 150, 'Australia'),  # Base - specific islands use suffixes
        
        # Antarctic
        ('KC4', 13, 'Antarctica'),
        ('DP0', 13, 'Antarctica'),  # German Antarctic
        ('RI1', 13, 'Antarctica'),  # Russian Antarctic
        ('VP8', 13, 'Antarctica'),  # Base - specifics use suffixes
        ('R1FJ', 61, 'Franz Josef Land'),
    ]
    
    existing_prefixes = {r['prefix'] for r in rules}
    
    for prefix, entity_id, comment in itu_expansions:
        if entity_id not in by_id:
            print(f"  WARNING: ITU expansion entity {entity_id} not found for {prefix}")
            continue
        
        if prefix not in existing_prefixes:
            priority = 10 + len(prefix) * 10
            rules.append({
                'prefix': prefix,
                'entity_id': entity_id,
                'priority': priority,
                'exact': False,
                'comment': comment,
            })
    
    return rules


def main():
    base = Path(__file__).parent.parent
    json_path = base / "src-tauri" / "resources" / "dxcc_entities.json"
    rs_path = base / "src-tauri" / "src" / "reference" / "prefixes.rs"
    output_path = base / "src-tauri" / "resources" / "prefix_rules.json"
    
    print("Loading DXCC entities...")
    by_id, by_name = load_entities(json_path)
    active_count = sum(1 for d in by_id.values() if not d['deleted'])
    print(f"  Loaded {len(by_id)} entities ({active_count} active)")
    
    print("\nParsing existing prefix rules...")
    existing_rules = parse_prefixes_rs(rs_path)
    print(f"  Found {len(existing_rules)} rules")
    
    print("\nBuilding corrected prefix rules...")
    rules, covered = build_prefix_rules(by_id, by_name, existing_rules)
    print(f"  Built {len(rules)} rules covering {len(covered)} entities")
    
    print("\nAdding disambiguation rules...")
    rules = add_disambiguation_rules(rules, by_id)
    
    print("\nAdding ITU block expansions...")
    rules = add_itu_expansions(rules, by_id)
    
    # Deduplicate and sort
    seen = set()
    unique_rules = []
    for r in rules:
        key = (r['prefix'], r['entity_id'])
        if key not in seen:
            seen.add(key)
            unique_rules.append(r)
    
    unique_rules.sort(key=lambda r: (r['prefix'], -r['priority']))
    
    # Convert all entity_id values to 3-digit zero-padded strings (ARRL format)
    for rule in unique_rules:
        rule['entity_id'] = f"{rule['entity_id']:03d}"
    
    # Recalculate coverage (use string format now)
    final_covered = {r['entity_id'] for r in unique_rules}
    active_entities = {f"{eid:03d}" for eid, d in by_id.items() if not d['deleted']}
    missing = active_entities - final_covered
    
    print(f"\nFinal statistics:")
    print(f"  Total rules: {len(unique_rules)}")
    print(f"  Entities covered: {len(final_covered)}/{len(active_entities)}")
    print(f"  Missing entities: {len(missing)}")
    
    if missing:
        print("\n  Still missing:")
        for eid in sorted(missing):
            # Convert string back to int for by_id lookup
            eid_int = int(eid)
            print(f"    {eid}: {by_id[eid_int]['name']} - prefixes: {by_id[eid_int].get('prefixes')}")
    
    # Build output
    output = {
        'version': '2.0.0',  # Bumped for string entity_id format
        'generated': datetime.now().isoformat(),
        'source': 'ARRL DXCC list + ITU Radio Regulations + operator conventions',
        'authority': 'https://www.arrl.org/files/file/DXCC/Current_Deleted.txt',
        'note': 'entity_id uses ARRL 3-digit zero-padded string format (e.g., "001" for Canada)',
        'stats': {
            'total_rules': len(unique_rules),
            'entities_covered': len(final_covered),
            'active_entities': len(active_entities),
            'coverage_percent': round(100 * len(final_covered) / len(active_entities), 1),
        },
        'rules': unique_rules,
    }
    
    with open(output_path, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"\nOutput written to: {output_path}")
    print(f"Coverage: {output['stats']['coverage_percent']}%")


if __name__ == "__main__":
    main()
