import os
import sys
from pathlib import Path
from datasets import load_dataset
from transformers import AutoTokenizer, AutoModelForSequenceClassification, Trainer, TrainingArguments

# ดึง path เพื่อให้อ่านไฟล์ config.py ได้ไม่ว่าจะรันจากโฟลเดอร์ไหน
sys.path.append(str(Path(__file__).parent))
from config import TrainingConfig, SynthConfig

def main():
    t_config = TrainingConfig()
    s_config = SynthConfig()
    
    data_path = str(s_config.output_path)
    labels = t_config.labels
    
    print(f"⏳ 1. กำลังโหลด Dataset จาก: {data_path}")
    # โหลดไฟล์ข้อมูล 5,000 บรรทัดที่คุณสร้างไว้
    dataset = load_dataset('json', data_files=data_path, split='train')
    
    # แปลง Text Label เป็นตัวเลข
    label2id = {label: i for i, label in enumerate(labels)}
    id2label = {i: label for i, label in enumerate(labels)}
    
    def map_labels(example):
        example['label'] = label2id[example['label']]
        return example
        
    dataset = dataset.map(map_labels)
    dataset = dataset.train_test_split(test_size=0.2, seed=t_config.seed) # แบ่งไปสอบ 20%
    
    print(f"✂️ 2. กำลังโหลด Tokenizer: {t_config.base_model}")
    tokenizer = AutoTokenizer.from_pretrained(t_config.base_model)
    
    def tokenize_function(examples):
        return tokenizer(examples["text"], padding="max_length", truncation=True, max_length=t_config.max_seq_length)
        
    tokenized_datasets = dataset.map(tokenize_function, batched=True)
    
    print("🧠 3. กำลังโหลดโมเดล...")
    model = AutoModelForSequenceClassification.from_pretrained(
        t_config.base_model, 
        num_labels=len(labels),
        id2label=id2label,
        label2id=label2id
    )
    
    print("🚀 4. เริ่มทำการเทรนโมเดล (Training)...")
    output_dir = str(t_config.output_dir)
    os.makedirs(output_dir, exist_ok=True)
    
    training_args = TrainingArguments(
        output_dir=output_dir,
        eval_strategy="epoch",        # วัดผลทุกๆ 1 รอบ
        save_strategy="epoch",
        learning_rate=t_config.learning_rate,
        per_device_train_batch_size=16, # ปรับ Batch Size ลงเพื่อป้องกัน RAM เครื่องเต็ม
        per_device_eval_batch_size=16,
        num_train_epochs=t_config.num_epochs,
        weight_decay=t_config.weight_decay,
        load_best_model_at_end=True,
    )
    
    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=tokenized_datasets["train"],
        eval_dataset=tokenized_datasets["test"],
    )
    
    trainer.train()
    
    print("💾 5. กำลังบันทึกโมเดล...")
    final_model_path = os.path.join(output_dir, "final_model")
    trainer.save_model(final_model_path)
    tokenizer.save_pretrained(final_model_path)
    print(f"🎉 เสร็จสิ้น! โมเดลของคุณฉลาดขึ้นและพร้อมใช้งานที่: {final_model_path}")

if __name__ == "__main__":
    main()