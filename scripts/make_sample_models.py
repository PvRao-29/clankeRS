#!/usr/bin/env python3
"""Generate the sample ONNX models and reference outputs shipped with clankeRS.

This is the *source of truth* for the artifacts under `sample_data/models/` and
the PyTorch reference output used by `clankers validate-model`.

Run it whenever you want to regenerate the sample models:

    pip install torch onnx
    python3 scripts/make_sample_models.py

It exports two small, deterministic PyTorch models to ONNX and records the
PyTorch reference output for the policy model so that `clankers validate-model`
can prove the Rust ONNX runtime reproduces PyTorch to within tolerance --
*without* requiring PyTorch to be installed at validation/CI time.
"""

import json
from pathlib import Path

import torch
import torch.nn as nn

REPO = Path(__file__).resolve().parent.parent
MODELS_DIR = REPO / "sample_data" / "models"
POLICY_INPUTS = REPO / "sample_data" / "policy_inputs"
DETECTOR_INPUTS = REPO / "sample_data" / "detector_inputs"

# Fixed seed => deterministic weights => stable, committable reference outputs.
SEED = 1234
OPSET = 17


class PolicyNet(nn.Module):
    """Tiny robot control policy: 10 state features -> 6 action dims."""

    def __init__(self, in_dim: int = 10, hidden: int = 32, out_dim: int = 6):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(in_dim, hidden),
            nn.ReLU(),
            nn.Linear(hidden, out_dim),
        )

    def forward(self, x):
        return self.net(x)


class SimpleClassifier(nn.Module):
    """Tiny camera detector: [1,3,224,224] image -> 6 class scores."""

    def __init__(self, num_classes: int = 6):
        super().__init__()
        self.features = nn.Sequential(
            nn.Conv2d(3, 16, 3, padding=1),
            nn.ReLU(),
            nn.AdaptiveAvgPool2d(1),
        )
        self.classifier = nn.Linear(16, num_classes)

    def forward(self, x):
        x = self.features(x)
        x = x.view(x.size(0), -1)
        return self.classifier(x)


def export(model: nn.Module, dummy: torch.Tensor, out_path: Path) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    model.eval()
    # Use the legacy TorchScript exporter for a stable, self-contained graph.
    torch.onnx.export(
        model,
        dummy,
        str(out_path),
        opset_version=OPSET,
        input_names=["input"],
        output_names=["output"],
        dynamo=False,
    )
    print(f"  exported {out_path.relative_to(REPO)}")


def main() -> None:
    torch.manual_seed(SEED)

    # --- Policy model (drives `clankers validate-model`) ---
    policy = PolicyNet()
    export(policy, torch.randn(1, 10), MODELS_DIR / "policy.onnx")

    # Reference output for the shipped sample input.
    sample = json.loads((POLICY_INPUTS / "input.json").read_text())
    x = torch.tensor(sample, dtype=torch.float32).reshape(1, -1)
    with torch.no_grad():
        y = policy(x)
    reference = y.flatten().tolist()
    (POLICY_INPUTS / "expected_output.json").write_text(json.dumps(reference) + "\n")
    print(f"  wrote sample_data/policy_inputs/expected_output.json = {reference}")

    # --- Camera detector (drives the camera_perception_node demo) ---
    torch.manual_seed(SEED + 1)
    detector = SimpleClassifier()
    export(detector, torch.randn(1, 3, 224, 224), MODELS_DIR / "detector.onnx")

    # Reference output for a deterministic, exactly-representable input
    # (an all-0.5 image). Rust reproduces the same input with `vec![0.5; N]`,
    # so `clankers validate-model` can diff PyTorch vs Rust with no stored
    # 150k-float input file.
    with torch.no_grad():
        y_det = detector(torch.full((1, 3, 224, 224), 0.5))
    det_reference = y_det.flatten().tolist()
    DETECTOR_INPUTS.mkdir(parents=True, exist_ok=True)
    (DETECTOR_INPUTS / "expected_output.json").write_text(
        json.dumps(det_reference) + "\n"
    )
    print(f"  wrote sample_data/detector_inputs/expected_output.json = {det_reference}")

    print("done.")


if __name__ == "__main__":
    main()
