---
title: "msadeqsirjani/dnd_ai_dm: An AI-powered Dungeon Master that generates dynamic D&D adventures, manages NPCs, and handles combat mechanics. Create immersive storytelling experiences with procedural world generation and natural language interaction"
source: "https://github.com/msadeqsirjani/dnd_ai_dm"
author:
published:
created: 2026-04-16
description: "An AI-powered Dungeon Master that generates dynamic D&D adventures, manages NPCs, and handles combat mechanics. Create immersive storytelling experiences with procedural world generation and natural language interaction - msadeqsirjani/dnd_ai_dm"
tags:
  - "clippings"
---
## 🎲 AI-Powered D&D Game Master 🐉

An intelligent Dungeon Master powered by AI that creates dynamic, engaging D&D adventures! This console-based application generates unique storylines, manages NPCs, and handles combat mechanics while adapting to player choices.

## ✨ Features

- 🤖 AI-driven storyline generation using GPT models
- 🎭 Dynamic NPC creation and management
- ⚔️ Turn-based combat system with D&D 5e rules
- 🌍 Procedural world generation
- 🎲 Realistic dice rolling mechanics
- 💬 Natural language processing for player commands

## 🏗️ Project Structure

```
dnd_ai_dm/
├── core/ # Core game mechanics
│ ├── game_manager.py # Main game controller
│ ├── story_generator.py # AI story generation
│ ├── npc_manager.py # NPC handling
│ └── combat_system.py # Combat mechanics
├── models/ # Game entities
│ ├── character.py # Player character
│ ├── npc.py # NPC definition
│ └── world_state.py # World tracking
├── utils/ # Utility functions
│ ├── dice.py # Dice rolling
│ └── text_processor.py # Text handling
└── main.py # Entry point
```

## 🚀 Getting Started

### Prerequisites

- Python 3.8+
- OpenAI API key

### Installation

1. **Clone the repository**
```
git clone https://github.com/msadeqsirjani/dnd_ai_dm.git
```
1. **Create a virtual environment**
```
python -m venv venv
```
1. **Activate the virtual environment**
```
source venv/bin/activate
```
1. **Install requirements**
```
pip install -r requirements.txt
```
1. **Set up environment variables**

Create a `.env` file in the project root:

```
OPENAI_API_KEY=<your-openai-api-key>
```

### 🎮 Running the Game

```
python main.py
```

## 🎯 Core Components

### 🤖 Story Generator

- Utilizes OpenAI's GPT models
- Creates dynamic quest narratives
- Adapts story based on player choices

### 🎭 NPC Manager

- Generates unique NPCs with personalities
- Manages NPC interactions and relationships
- Tracks NPC locations and knowledge

### ⚔️ Combat System

- Initiative-based turn order
- D&D 5e combat rules
- Dice rolling mechanics

### 🌍 World State

- Tracks game world status
- Manages locations and quests
- Handles time and weather systems

## 📝 Example Gameplay

```
Welcome to AI D&D!
=================
Your quest begins:
You find yourself in the village of Eldermist, where rumors of strange
disappearances have been circulating...
What would you like to do?: talk to innkeeper
```