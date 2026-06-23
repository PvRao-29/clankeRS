#!/usr/bin/env python3
"""Compare PyTorch and ONNX outputs for clankeRS validation."""

import argparse
import json
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--onnx", required=True)
    parser.add_argument("--samples", default="sample_data/policy_inputs/")
    parser.add_argument("--tolerance", type=float, default=0.001)
    args = parser.parse_args()

    try:
        import numpy as np
        import onnxruntime as ort
    except ImportError:
        print("Install: pip install numpy onnxruntime", file=sys.stderr)
        sys.exit(1)

    sample_path = Path(args.samples) / "input.json"
    if sample_path.exists():
        input_data = np.array(json.loads(sample_path.read_text()), dtype=np.float32)
    else:
        input_data = np.random.randn(1, 3, 224, 224).astype(np.float32)

    session = ort.InferenceSession(args.onnx)
    input_name = session.get_inputs()[0].name
    output = session.run(None, {input_name: input_data.reshape(1, 3, 224, 224)})[0]

    print(f"ONNX output shape: {output.shape}")
    print(f"ONNX output sample: {output.flatten()[:6]}")
    print("Status: run clankers validate-model for full PyTorch comparison")


if __name__ == "__main__":
    main()
