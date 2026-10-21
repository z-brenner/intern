"""Inputs, outputs, opsets and operator counts of ONNX models.
usage: python -I dump_onnx.py MODEL.onnx..."""
import sys, onnx
from onnx import TensorProto
for p in sys.argv[1:]:
    m = onnx.load(p, load_external_data=False)
    ops = ", ".join(f"{o.domain or 'ai.onnx'}:{o.version}" for o in m.opset_import)
    print(f"## {p.split('/')[-1]}  ir={m.ir_version} opsets=[{ops}] producer={m.producer_name} {m.producer_version} nodes={len(m.graph.node)}")
    if m.metadata_props: print("  metadata:", {x.key: x.value[:200] for x in m.metadata_props})
    inits = {i.name for i in m.graph.initializer}
    def fmt(v):
        t = v.type.tensor_type
        dims = [d.dim_param if d.dim_param else d.dim_value for d in t.shape.dim]
        return f"{v.name}: {TensorProto.DataType.Name(t.elem_type)} {dims}"
    for v in m.graph.input:
        if v.name not in inits: print("  in ", fmt(v))
    for v in m.graph.output: print("  out", fmt(v))
    from collections import Counter
    print("  ops:", dict(Counter(n.op_type for n in m.graph.node).most_common(40)))
