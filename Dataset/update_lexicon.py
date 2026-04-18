import json
import os

def update_lexicon_with_spells():
    # 1. โหลดไฟล์ Lexicon เดิมที่สกัดจาก Pandas
    lexicon_file = 'dnd_lexicon.json'
    if os.path.exists(lexicon_file):
        with open(lexicon_file, 'r', encoding='utf-8') as f:
            lexicon = json.load(f)
    else:
        lexicon = {}
        print("ไม่พบไฟล์ dnd_lexicon.json เดิม จะสร้างขึ้นมาใหม่")

    # 2. นำข้อมูลจาก Pinebrook (Option 2) + 5e Common Spells (Inspire from Notebook)
    spells_list = [
        # ---- Cantrips ----
        "Acid Splash", "Blade Ward", "Booming Blade", "Chill Touch", "Control Flames",
        "Create Bonfire", "Dancing Lights", "Druidcraft", "Eldritch Blast", "Fire Bolt",
        "Friends", "Frostbite", "Green-Flame Blade", "Guidance", "Gust", "Infestation",
        "Light", "Mage Hand", "Magic Stone", "Mending", "Message", "Minor Illusion",
        "Mold Earth", "Poison Spray", "Prestidigitation", "Primal Savagery", "Produce Flame",
        "Ray of Frost", "Resistance", "Sacred Flame", "Sapping Sting", "Shape Water",
        "Shillelagh", "Shocking Grasp", "Spare the Dying", "Sword Burst", "Thaumaturgy",
        "Thorn Whip", "Thunderclap", "Toll the Dead", "True Strike", "Vicious Mockery",
        "Word of Radiance",
        # ---- Level 1 ----
        "Absorb Elements", "Alarm", "Animal Friendship", "Armor of Agathys", "Arms of Hadar",
        "Bane", "Bless", "Burning Hands", "Catapult", "Cause Fear", "Ceremony",
        "Chaos Bolt", "Charm Person", "Chromatic Orb", "Color Spray", "Command",
        "Compelled Duel", "Comprehend Languages", "Create or Destroy Water", "Cure Wounds",
        "Detect Evil and Good", "Detect Magic", "Detect Poison and Disease",
        "Disguise Self", "Dissonant Whispers", "Divine Favor", "Earth Tremor",
        "Ensnaring Strike", "Entangle", "Expeditious Retreat", "Faerie Fire",
        "False Life", "Feather Fall", "Find Familiar", "Fog Cloud", "Goodberry",
        "Grease", "Guiding Bolt", "Hail of Thorns", "Healing Word", "Hellish Rebuke",
        "Heroism", "Hex", "Hunter's Mark", "Identify", "Illusory Script",
        "Inflict Wounds", "Jump", "Longstrider", "Mage Armor", "Magic Missile",
        "Protection from Evil and Good", "Purify Food and Drink", "Ray of Sickness",
        "Sanctuary", "Searing Smite", "Shield", "Shield of Faith", "Silent Image",
        "Sleep", "Speak with Animals", "Tasha's Hideous Laughter", "Tenser's Floating Disk",
        "Thunderous Smite", "Thunderwave", "Unseen Servant", "Witch Bolt", "Wrathful Smite",
        # ---- Level 2 ----
        "Aid", "Alter Self", "Animal Messenger", "Arcane Lock", "Augury",
        "Barkskin", "Beast Sense", "Blindness/Deafness", "Blur", "Branding Smite",
        "Calm Emotions", "Cloud of Daggers", "Continual Flame", "Cordon of Arrows",
        "Crown of Madness", "Darkness", "Darkvision", "Detect Thoughts", "Dragon's Breath",
        "Enhance Ability", "Enlarge/Reduce", "Enthrall", "Find Steed", "Find Traps",
        "Flame Blade", "Flaming Sphere", "Gentle Repose", "Gust of Wind", "Healing Spirit",
        "Heat Metal", "Hold Person", "Invisibility", "Kinetic Jaunt", "Knock",
        "Lesser Restoration", "Levitate", "Locate Animals or Plants", "Locate Object",
        "Magic Mouth", "Magic Weapon", "Melf's Acid Arrow", "Mind Spike", "Mirror Image",
        "Misty Step", "Moonbeam", "Nystul's Magic Aura", "Pass without Trace",
        "Phantasmal Force", "Prayer of Healing", "Protection from Poison", "Ray of Enfeeblement",
        "Rope Trick", "Scorching Ray", "See Invisibility", "Shatter", "Silence",
        "Skywrite", "Spider Climb", "Spike Growth", "Spiritual Weapon", "Suggestion",
        "Summon Beast", "Warding Bond", "Warding Wind", "Web", "Zone of Truth",
        # ---- Level 3 ----
        "Animate Dead", "Aura of Vitality", "Beacon of Hope", "Bestow Curse",
        "Blinding Smite", "Blink", "Call Lightning", "Catnap", "Clairvoyance",
        "Conjure Animals", "Conjure Barrage", "Counterspell", "Create Food and Water",
        "Crusader's Mantle", "Daylight", "Dispel Magic", "Elemental Weapon",
        "Enemies Abound", "Erupting Earth", "Fast Friends", "Fear", "Feign Death",
        "Fireball", "Flame Arrows", "Fly", "Gaseous Form", "Glyph of Warding",
        "Haste", "Hunger of Hadar", "Hypnotic Pattern", "Leomund's Tiny Hut",
        "Life Transference", "Lightning Arrow", "Lightning Bolt", "Magic Circle",
        "Major Image", "Mass Healing Word", "Meld into Stone", "Melf's Minute Meteors",
        "Nondetection", "Phantom Steed", "Plant Growth", "Protection from Energy",
        "Remove Curse", "Revivify", "Sending", "Sleet Storm", "Slow", "Spirit Guardians",
        "Spirit Shroud", "Stinking Cloud", "Summon Fey", "Summon Lesser Demons",
        "Summon Shadowspawn", "Summon Undead", "Thunder Step", "Tidal Wave",
        "Tongues", "Vampiric Touch", "Wall of Sand", "Wall of Water", "Water Breathing",
        "Water Walk", "Wind Wall",
        # ---- Level 4 ----
        "Arcane Eye", "Aura of Life", "Aura of Purity", "Banishment", "Black Tentacles",
        "Blight", "Charm Monster", "Compulsion", "Confusion", "Conjure Minor Elementals",
        "Conjure Woodland Beings", "Control Water", "Death Ward", "Dimension Door",
        "Divination", "Dominate Beast", "Elemental Bane", "Evard's Black Tentacles",
        "Fabricate", "Fire Shield", "Fount of Moonlight", "Freedom of Movement",
        "Giant Insect", "Grasping Vine", "Greater Invisibility", "Guardian of Faith",
        "Hallucinatory Terrain", "Ice Storm", "Leomund's Secret Chest", "Locate Creature",
        "Mordenkainen's Faithful Hound", "Mordenkainen's Private Sanctum", "Otiluke's Resilient Sphere",
        "Phantasmal Killer", "Polymorph", "Shadow of Moil", "Sickening Radiance",
        "Staggering Smite", "Stone Shape", "Stoneskin", "Storm Sphere", "Summon Aberration",
        "Summon Construct", "Summon Elemental", "Summon Greater Demon", "Vitriolic Sphere",
        "Wall of Fire",
        # ---- Level 5 ----
        "Animate Objects", "Antilife Shell", "Awaken", "Banishing Smite", "Bigby's Hand",
        "Circle of Power", "Cloudkill", "Commune", "Commune with Nature", "Cone of Cold",
        "Conjure Elemental", "Conjure Volley", "Contact Other Plane", "Contagion",
        "Creation", "Danse Macabre", "Dawn", "Destructive Wave", "Dispel Evil and Good",
        "Dominate Person", "Dream", "Enervation", "Far Step", "Flame Strike",
        "Geas", "Greater Restoration", "Hallow", "Hold Monster", "Holy Weapon",
        "Immolation", "Infernal Calling", "Insect Plague", "Legend Lore", "Mass Cure Wounds",
        "Maelstrom", "Mislead", "Modify Memory", "Negative Energy Flood", "Passwall",
        "Planar Binding", "Rary's Telepathic Bond", "Raise Dead", "Reincarnate",
        "Scrying", "Seeming", "Skill Empowerment", "Steel Wind Strike", "Summon Celestial",
        "Swift Quiver", "Telekinesis", "Teleportation Circle", "Tree Stride",
        "Wall of Force", "Wall of Light", "Wall of Stone", "Wrath of Nature",
        # ---- Level 6 ----
        "Arcane Gate", "Blade Barrier", "Bones of the Earth", "Chain Lightning",
        "Circle of Death", "Conjure Fey", "Contingency", "Create Homunculus", "Create Undead",
        "Disintegrate", "Drawmij's Instant Summons", "Druid Grove", "Eyebite",
        "Find the Path", "Flesh to Stone", "Forbiddance", "Globe of Invulnerability",
        "Guards and Wards", "Harm", "Heal", "Heroes' Feast", "Investiture of Flame",
        "Investiture of Ice", "Investiture of Stone", "Investiture of Wind",
        "Magic Jar", "Mass Suggestion", "Mental Prison", "Move Earth", "Otiluke's Freezing Sphere",
        "Otto's Irresistible Dance", "Planar Ally", "Primordial Ward", "Programmed Illusion",
        "Scatter", "Soul Cage", "Summon Fiend", "Sunbeam", "Tasha's Otherworldly Guise",
        "Tenser's Transformation", "Transport via Plants", "True Seeing", "Wall of Ice",
        "Wall of Thorns", "Wind Walk", "Word of Recall",
        # ---- Level 7 ----
        "Conjure Celestial", "Crown of Stars", "Delayed Blast Fireball", "Divine Word",
        "Dream of the Blue Veil", "Etherealness", "Finger of Death", "Fire Storm",
        "Forcecage", "Mirage Arcane", "Mordenkainen's Magnificent Mansion",
        "Mordenkainen's Sword", "Plane Shift", "Power Word Pain", "Prismatic Spray",
        "Project Image", "Regenerate", "Resurrection", "Reverse Gravity", "Sequester",
        "Simulacrum", "Symbol", "Temple of the Gods", "Teleport", "Whirlwind",
        # ---- Level 8 ----
        "Abi-Dalzim's Horrid Wilting", "Animal Shapes", "Antimagic Field", "Antipathy/Sympathy",
        "Clone", "Control Weather", "Demiplane", "Dominate Monster", "Earthquake",
        "Feeblemind", "Glibness", "Holy Aura", "Illusory Dragon", "Incendiary Cloud",
        "Maze", "Mind Blank", "Power Word Stun", "Sunburst", "Telepathy", "Tsunami",
        # ---- Level 9 ----
        "Astral Projection", "Blade of Disaster", "Foresight", "Gate", "Imprisonment",
        "Invulnerability", "Mass Heal", "Mass Polymorph", "Meteor Swarm", "Power Word Heal",
        "Power Word Kill", "Prismatic Wall", "Psychic Scream", "Shapechange",
        "Storm of Vengeance", "Time Stop", "True Polymorph", "True Resurrection",
        "Weird", "Wish",
    ]

    # 3. รวมกับ spells เดิม (ไม่เขียนทับ) — เผื่ออนาคตเพิ่มเข้าไปอีก
    existing = set(lexicon.get("spells", []))
    before = len(existing)
    existing.update(spells_list)
    lexicon["spells"] = sorted(existing)
    added = len(existing) - before

    # 4. เซฟทับไฟล์เดิม
    with open(lexicon_file, 'w', encoding='utf-8') as f:
        json.dump(lexicon, f, indent=4, ensure_ascii=False)

    print(f"✨ อัปเดตสำเร็จ! เพิ่มใหม่ {added} คาถา (รวมทั้งหมด {len(lexicon['spells'])} คาถา)")

if __name__ == "__main__":
    update_lexicon_with_spells()