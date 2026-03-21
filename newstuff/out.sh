#!/usr/bin/env wolframscript
NetVisualize[net_] :=  EdgeTaggedGraph[Range[Length[net]], Join @@ Table[Table[DirectedEdge[nodei, edge[[2]], edge[[1]]], {edge, net[[nodei]]}], {nodei, Length[net]}], VertexLabels -> "Name", EdgeLabels -> "EdgeTag", ImageSize -> Large]
Export["renders/0AllNet0.png", Labeled[NetVisualize[{{0->1, 1->1}}], "0AllNet0"]];
Export["renders/1stepped0.png", Labeled[NetVisualize[{{0->1,0->2}, {0->3, 1->4}, {0->5, 1->6}, {0->8, 1->7}, {0->9, 1->10}, {0->12, 1->11}, {0->14, 1->13}, {0->15, 1->16}, {0->1, 1->2}, {0->4, 1->3}, {0->6, 1->5}, {0->7, 1->8}, {0->10, 1->9}, {0->11, 1->12}, {0->13, 1->14}, {0->16, 1->15}}], "1stepped0"]];
Export["renders/2withoutunreachable0.png", Labeled[NetVisualize[{{0->1,0->2}, {0->3, 1->4}, {0->5, 1->6}, {0->8, 1->7}, {0->9, 1->10}, {0->12, 1->11}, {0->14, 1->13}, {0->15, 1->16}, {0->1, 1->2}, {0->4, 1->3}, {0->6, 1->5}, {0->7, 1->8}, {0->10, 1->9}, {0->11, 1->12}, {0->13, 1->14}, {0->16, 1->15}}], "2withoutunreachable0"]];
