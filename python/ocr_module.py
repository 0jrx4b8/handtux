from transformers import TrOCRProcessor, VisionEncoderDecoderModel
from PIL import Image
import io

class OCREngine:
    def __init__(self):
        self.processor = TrOCRProcessor.from_pretrained("microsoft/trocr-base-handwritten")
        self.model = VisionEncoderDecoderModel.from_pretrained("microsoft/trocr-base-handwritten")
    
    def process_image(self, image_bytes: bytes) -> list[str]:
        """Process image bytes and return text candidates"""
        image = Image.open(io.BytesIO(image_bytes)).convert("RGB")
        pixel_values = self.processor(image, return_tensors="pt").pixel_values
        generated_ids = self.model.generate(pixel_values)
        return [self.processor.batch_decode(generated_ids, skip_special_tokens=True)[0]]

# Singleton instance
_engine = OCREngine()

def get_engine():
    return _engine