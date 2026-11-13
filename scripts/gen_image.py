#!/usr/bin/env python3
"""
一键免费 AI 生图工具 (基于 Pollinations.ai / HuggingFace 无需任何 Key)
"""

import sys
import os
import urllib.parse
import urllib.request

def generate_image(prompt, out_path, model="flux", width=1024, height=1024, seed=42):
    encoded_prompt = urllib.parse.quote(prompt)
    url = f"https://image.pollinations.ai/prompt/{encoded_prompt}?width={width}&height={height}&nologo=true&seed={seed}&model={model}"
    
    os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
    print(f"🎨 正在调用免费 AI 生图引擎 (模型: {model})...")
    print(f"🔗 请求地址: {url[:80]}...")
    
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req) as resp, open(out_path, "wb") as f:
        f.write(resp.read())
        
    print(f"✅ 生成成功！文件已保存至: {out_path} ({os.path.getsize(out_path)} 字节)")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("用法: python3 gen_image.py <提示词> <输出图片路径>")
        sys.exit(1)
    prompt = sys.argv[1]
    out_file = sys.argv[2]
    generate_image(prompt, out_file)
