#!/usr/bin/env python3
import pandas as pd
import re
import json

def extract_dnd_vocab():
    print("⏳ กำลังโหลดไฟล์ข้อมูล...")
    try:
        df_monsters_1 = pd.read_csv('cleaned_monsters_basic.csv')
        df_monsters_2 = pd.read_csv('aidedd_blocks2.csv')
        df_items = pd.read_csv('roll_20_items.csv')
    except FileNotFoundError as e:
        print(f"❌ หาไฟล์ไม่พบ: {e}")
        return

    # 1. สกัดชื่อมอนสเตอร์ (Clean Monster Names)
    print("🧹 กำลังทำความสะอาดชื่อมอนสเตอร์...")
    monsters = set(df_monsters_1['name'].dropna().str.lower().str.title())
    monsters.update(df_monsters_2['name'].dropna().str.lower().str.title())
    # กรองเอาพวกคำอธิบายในวงเล็บออก และจำกัดความยาวไม่ให้เป็นประโยคยาว
    clean_monsters = [m for m in monsters if len(m) < 30 and "(" not in m]

    # 2. สกัดชื่อไอเทมและอาวุธ (Clean Items & Weapons)
    print("⚔️ กำลังคัดแยกอาวุธและไอเทม...")
    items = df_items['item'].dropna().str.lower().str.title().tolist()
    clean_items = [i for i in items if len(i) < 30 and "(" not in i]
    # แยกอาวุธออกมาต่างหาก เพื่อเอาไว้สร้างประโยคโจมตี
    weapons = df_items[df_items['type'].str.contains('weapon', case=False, na=False)]['item'].dropna().str.lower().str.title().tolist()
    
    # 3. สกัดชื่อท่าโจมตีของมอนสเตอร์ (Extract Actions/Attacks)
    # คอลัมน์ actions ปกติจะมาเป็นก้อนยาวๆ เช่น "Talon. Melee Weapon Attack: +4 to hit..."
    print("🔥 กำลังใช้ Regex แกะรอยท่าโจมตี...")
    actions_raw = df_monsters_2['actions'].dropna()
    action_names = set()
    
    for raw_action in actions_raw:
        parts = str(raw_action).split('|') # แยกท่าต่างๆ ด้วย |
        for part in parts:
            part = part.strip()
            name = part.split('.')[0].strip() # เอาเฉพาะคำก่อนจุด Full Stop แรก
            
            # กรองเงื่อนไข: สั้นกว่า 25 ตัวอักษร, ไม่ใช่ตัวพิมพ์เล็กทั้งหมด
            if len(name) < 25 and not name.islower():
                # ลบคำสั่งแปลกๆ ในวงเล็บทิ้ง เช่น (Recharge 5-6)
                name = re.sub(r'\(.*?\)', '', name).strip()
                action_names.add(name)
                
    # กรองคำสั่งทั่วๆ ไปที่หลุดมาทิ้ง
    clean_actions = [a.title() for a in action_names if "recharge" not in a.lower() and "day" not in a.lower() and a != ""]

    # 3.5 สกัดชื่อ Spell จาก attributes (Spellcasting / Innate Spellcasting)
    print("✨ กำลังแกะรอย Spells...")
    spell_names = set()
    spell_slot_re = re.compile(
        r'(?:cantrips?\s*\(at will\)|\d+(?:st|nd|rd|th)\s*level\s*\([^)]*\)|at will|\d+\s*/\s*day(?:\s*each)?)\s*:\s*([^|]+)',
        re.IGNORECASE,
    )
    for attr in df_monsters_2['attributes'].dropna():
        for match in spell_slot_re.finditer(str(attr)):
            for raw in match.group(1).split(','):
                name = re.sub(r'\([^)]*\)', '', raw).strip().strip('.').strip()
                if 2 < len(name) < 40 and not any(ch.isdigit() for ch in name):
                    spell_names.add(name.title())

    # 4. รวมเป็น Lexicon Dictionary
    lexicon = {
        "monsters": sorted(list(clean_monsters)),
        "items": sorted(list(set(clean_items))),
        "weapons": sorted(list(set(weapons))),
        "monster_actions": sorted(list(set(clean_actions))),
        "spells": sorted(list(spell_names)),
    }

    # 5. บันทึกผลลัพธ์เป็น JSON ที่สะอาดและเบาบาง
    with open('dnd_lexicon.json', 'w', encoding='utf-8') as f:
        json.dump(lexicon, f, indent=4, ensure_ascii=False)
        
    print("\n✅ ดึงข้อมูลสำเร็จ! ผลลัพธ์ที่ได้:")
    print(f"🐉 พบมอนสเตอร์: {len(lexicon['monsters'])} ตัว")
    print(f"🎒 พบไอเทม: {len(lexicon['items'])} ชิ้น")
    print(f"🗡️ พบอาวุธ: {len(lexicon['weapons'])} ชิ้น")
    print(f"💥 พบท่าโจมตี: {len(lexicon['monster_actions'])} ท่า")
    print(f"✨ พบคาถา: {len(lexicon['spells'])} คาถา")
    print("\n📁 บันทึกข้อมูลพร้อมใช้ลงในไฟล์ 'dnd_lexicon.json' เรียบร้อยแล้ว")

if __name__ == "__main__":
    extract_dnd_vocab()