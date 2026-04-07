# Volunteering and Community

## Activity 1

- **Organisation / Community**: tinygrad (open source deep learning framework)
- **Role**: Open source contributor
- **Period**: 2025
- **Description**: Implemented the ONNX LSTM operator in `tinygrad/nn/onnx.py`, adding support for forward, reverse, and bidirectional modes, optional inputs including initial hidden and cell states, and a shared activation helper covering the full RNN activation set required by the ONNX specification. Added comprehensive regression tests in `test/external/external_test_onnx_ops.py` validating numerical correctness against ONNX Runtime across forward, reverse, bidirectional, and stateful variants. Submitted as PR #15453 (https://github.com/tinygrad/tinygrad/pull/15453).
- **Impact**: Demonstrated end-to-end fluency with the tinygrad codebase, the ONNX specification, and reference-runtime regression testing methodology. PR was closed for exceeding tinygrad's strict line-count guidelines rather than for technical incorrectness.