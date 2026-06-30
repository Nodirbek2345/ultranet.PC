from PIL import Image

def convert_to_ico():
    input_path = 'C:/Users/AI/.gemini/antigravity-ide/brain/f0940ecc-039e-4c50-9bb3-52e44e02bab7/media__1782319509072.jpg'
    output_path = 'e:/Loyixalarim/UltraNet AI/ultranet/windows/runner/resources/app_icon.ico'
    try:
        img = Image.open(input_path)
        img.save(output_path, format='ICO', sizes=[(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)])
        print("Success")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    convert_to_ico()
