"""
Extracts binary image data from SAX-NeRF pickle files.

Usage: python3 teapot_50.pickle
"""

import sys
import pickle as pkl

import numpy as np
import skimage.io

path = sys.argv[1]
with open(path, "rb") as file:
    data = pkl.load(file)

image = data["image"]
print(f"Shape: {image.shape}")

output_path = path.replace(".pickle", ".bin")
image.tofile(output_path)
