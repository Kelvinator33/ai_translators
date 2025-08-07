#!/usr/bin/env python3
from PIL import Image

# Create a simple test image
img = Image.new('RGB', (300, 200), color='white')
img.save('test_image.png')
print("Test image created: test_image.png")