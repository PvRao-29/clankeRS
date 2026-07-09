#!/usr/bin/env python3
"""Generate ONNX test fixtures for clankers-ml integration tests.

Run from the repo root:

    pip install torch onnx
    python3 scripts/make_onnx_fixtures.py

Artifacts land in `crates/clankers-ml/tests/fixtures/onnx/`.
"""

from __future__ import annotations

from pathlib import Path

import torch
import torch.nn as nn

REPO = Path(__file__).resolve().parent.parent
FIXTURES = REPO / "crates" / "clankers-ml" / "tests" / "fixtures" / "onnx"
OPSET = 17
SEED = 4242


def export(model: nn.Module, args, path: Path, **kwargs) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    model.eval()
    torch.onnx.export(
        model,
        args,
        str(path),
        opset_version=OPSET,
        dynamo=False,
        **kwargs,
    )
    print(f"  exported {path.relative_to(REPO)}")


class PolicySingle(nn.Module):
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return x @ torch.tensor([[1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.5, 0.5]])


class MultiInputPolicy(nn.Module):
    def forward(self, image: torch.Tensor, state: torch.Tensor) -> torch.Tensor:
        img_feat = image.to(torch.float32).mean()
        return state + img_feat


class IdentityModule(nn.Module):
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return x


class DynamicBatchIdentity(nn.Module):
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return x


class TwoInputsOneOutput(nn.Module):
    def forward(self, a: torch.Tensor, b: torch.Tensor) -> torch.Tensor:
        return a + b


class OneInputTwoOutputs(nn.Module):
    def forward(self, x: torch.Tensor):
        return x, x * 2


def main() -> None:
    torch.manual_seed(SEED)
    FIXTURES.mkdir(parents=True, exist_ok=True)

    export(
        PolicySingle(),
        torch.randn(1, 4),
        FIXTURES / "policy_single_f32.onnx",
        input_names=["input"],
        output_names=["output"],
    )

    export(
        MultiInputPolicy(),
        (torch.randint(0, 255, (1, 64, 64, 3), dtype=torch.uint8), torch.randn(1, 12)),
        FIXTURES / "policy_multi_input_image_state.onnx",
        input_names=["image", "state"],
        output_names=["action"],
    )

    export(
        IdentityModule(),
        torch.randint(-5, 5, (1, 4), dtype=torch.int64),
        FIXTURES / "identity_i64.onnx",
        input_names=["input"],
        output_names=["output"],
    )

    export(
        IdentityModule(),
        torch.randint(0, 255, (1, 8), dtype=torch.uint8),
        FIXTURES / "identity_u8.onnx",
        input_names=["input"],
        output_names=["output"],
    )

    export(
        DynamicBatchIdentity(),
        torch.randn(2, 3),
        FIXTURES / "dynamic_batch_identity.onnx",
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={"input": {0: "batch"}, "output": {0: "batch"}},
    )

    export(
        TwoInputsOneOutput(),
        (torch.randn(1, 4), torch.randn(1, 4)),
        FIXTURES / "two_inputs_one_output.onnx",
        input_names=["a", "b"],
        output_names=["sum"],
    )

    export(
        OneInputTwoOutputs(),
        torch.randn(1, 3),
        FIXTURES / "one_input_two_outputs.onnx",
        input_names=["input"],
        output_names=["out_a", "out_b"],
    )

    print("done.")


if __name__ == "__main__":
    main()
