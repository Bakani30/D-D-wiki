import os
import requests
from pathlib import Path
from fastapi import FastAPI
from pydantic import BaseModel
import torch
from transformers import AutoTokenizer, AutoModelForSequenceClassification

BASE_DIR = Path(__file__).resolve().parent
MODEL_PATH = BASE_DIR / "models" / "checkpoints" / "final_model"

print("⏳ กำลังโหลดโมเดล Intent Classifier (MiniLM)...")
try:
    tokenizer = AutoTokenizer.from_pretrained(str(MODEL_PATH))
    model = AutoModelForSequenceClassification.from_pretrained(str(MODEL_PATH))
    print("✅ โหลด MiniLM สำเร็จ!")
except Exception as e:
    print(f"❌ โหลด MiniLM ไม่สำเร็จ: {e}")

app = FastAPI(title="Pinebrook Local AI", description="Local AI for D&D (Intent + Suggestions)")

class ChatInput(BaseModel):
    text: str

class SceneInput(BaseModel):
    scene_description: str

@app.post("/api/intent")
def get_intent(chat: ChatInput):
    inputs = tokenizer(chat.text, return_tensors="pt", truncation=True, max_length=128)
    with torch.no_grad():
        outputs = model(**inputs)
    probs = torch.nn.functional.softmax(outputs.logits, dim=-1)
    confidence, predicted_id = torch.max(probs, dim=-1)
    intent_label = model.config.id2label[predicted_id.item()]
    
    return {
        "text": chat.text,
        "intent": intent_label,
        "confidence": round(confidence.item(), 4)
    }

@app.post("/api/suggest")
def get_suggestions(scene: SceneInput):
    # สั่งให้ Ollama อ่านฉากแล้วคิดตัวเลือก
    prompt = f"""
    คุณคือ Dungeon Master ของเกม D&D 
    จากเนื้อเรื่องด้านล่างนี้ ให้คุณเสนอทางเลือกที่ผู้เล่นสามารถทำได้ 3 ข้อ (เขียนให้สั้น กระชับ และเป็นภาษาไทย)
    ต้องส่งกลับมาในรูปแบบ JSON Array ของ String เท่านั้น ห้ามมีข้อความอื่นปน
    
    เนื้อเรื่อง: {scene.scene_description}
    """
    
    try:
        response = requests.post('http://localhost:11434/api/generate', json={
            "model": "llama3.2",
            "prompt": prompt,
            "format": "json",
            "stream": False
        })
        result = response.json()
        
        import json
        raw_text = result['response']
        
        # ดักจับรูปแบบข้อมูลที่ Ollama อาจจะส่งมา
        try:
            parsed_data = json.loads(raw_text)
            if isinstance(parsed_data, dict):
                # ถ้ามาเป็น {"1": "...", "2": "..."} ให้ดึงมาแค่ค่า Value
                suggestions = list(parsed_data.values())
            elif isinstance(parsed_data, list):
                # ถ้ามาเป็น Array ปกติ ก็ใช้ได้เลย
                suggestions = parsed_data
            else:
                suggestions = ["สำรวจพื้นที่", "เตรียมอาวุธ", "ระวังตัว"]
        except:
            suggestions = ["เกิดข้อผิดพลาดในการอ่านตัวเลือก", "เตรียมพร้อม", "ตั้งรับ"]

        return {"suggestions": suggestions}
    except Exception as e:
        return {"error": str(e), "suggestions": ["สำรวจพื้นที่", "เตรียมอาวุธ", "ระวังตัว"]}
    
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8000)