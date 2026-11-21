import { Graphviz } from "@hpcc-js/wasm-graphviz";
import fs from "fs";
const gv = await Graphviz.load();
const [,, inp, out] = process.argv;
fs.writeFileSync(out, gv.dot(fs.readFileSync(inp, "utf8"), "svg"));
