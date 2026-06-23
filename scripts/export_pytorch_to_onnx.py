#!/usr/bin/env python3
"""Export a PyTorch model to ONNX for clankeRS deployment."""

import argparse
import importlib.util
import sys
from pathlib import Path


def load_model_from_path(spec: str):
    path, _, class_name = spec.partition(":")
    path = Path(path)
    spec = importlib.util.spec_from_file_location("user_model", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return getattr(module, class_name)


def main():
    parser = argparse.ArgumentParser(description="Export PyTorch model to ONNX")
    parser.add_argument("--model", required=True, help="path:ClassName")
    parser.add_argument("--checkpoint", required=True, help="path to .pt checkpoint")
    parser.add_argument("--output", required=True, help="output .onnx path")
    parser.add_argument("--opset", type=int, default=17)
    parser.add_argument("--input-shape", type=int, nargs="+", default=[1, 3, 224, 224])
    args = parser.parse_args()

    try:
        import torch
    except ImportError:
        print("PyTorch required: pip install torch", file=sys.stderr)
        sys.exit(1)

    ModelClass = load_model_from_path(args.model)
    model = ModelClass()
    state = torch.load(args.checkpoint, map_location="cpu", weights_only=True)
    if isinstance(state, dict) and "state_dict" in state:
        model.load_state_dict(state["state_dict"])
    else:
        model.load_state_dict(state)
    model.eval()

    dummy = torch.randn(*args.input_shape)
    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    torch.onnx.export(
        model,
        dummy,
        str(out_path),
        opset_version=args.opset,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes=None,
    )
    print(f"Exported ONNX model to {out_path}")


if __name__ == "__main__":
    main()
