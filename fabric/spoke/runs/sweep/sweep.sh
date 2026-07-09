#!/bin/bash
# Map every testing/ knowledge base back to SPOKE via bokf export -> spoke-fabric link.
# Captures per-KB stats + any errors/anomalies. Fail-soft: continues past a broken KB.
set -u
BOKF=/Users/wgu/Desktop/BioOKF/app/target/release/bokf
FABRIC=/Users/wgu/Desktop/BioOKF/fabric/spoke/target/release/spoke-fabric
T=/Users/wgu/Desktop/BioOKF/testing
OUT=/Users/wgu/Desktop/BioOKF/fabric/spoke/runs/sweep
mkdir -p "$OUT"

KBS=(
  "$T/benchmark-bioasq/01-glioblastoma-neuro-oncology"
  "$T/benchmark-bioasq/02-oncology-other"
  "$T/benchmark-bioasq/03-genetics-genomics-biobank"
  "$T/benchmark-bioasq/04-gene-regulation-epigenetics"
  "$T/benchmark-bioasq/05-pharmacology-therapeutics"
  "$T/benchmark-bioasq/06-infectious-disease-micro"
  "$T/benchmark-bioasq/07-neuromuscular-neuro"
  "$T/benchmark-bioasq/08-metabolic-cardio-renal"
  "$T/benchmark-bioasq/09-misc-clinical"
  "$T/benchmark-biored"
  "$T/demo-ebv-ms/ebv-ms"
  "$T/demo-ebv-ms/ebv-ms-viz"
)

echo "start $(date +%H:%M:%S)"
for kb in "${KBS[@]}"; do
  name=$(basename "$kb")
  exp="$OUT/$name-export.json"
  ann="$OUT/$name-annotated.json"
  rep="$OUT/$name-report.json"
  errlog="$OUT/$name.err"
  echo "---- $name ----"
  # 1. export (bokf CLI)
  if ! "$BOKF" export "$kb" --out "$exp" --name "$name" >"$errlog" 2>&1; then
    echo "  EXPORT FAILED (see $errlog)"; continue
  fi
  # normalize declared counts to the graph (bokf edge_count excludes synthesized edges -> finding)
  python3 - "$exp" <<'PY'
import json,sys
p=sys.argv[1]; e=json.load(open(p))
e["_declared_edge_count"]=e.get("edge_count"); e["_graph_edges"]=len(e["graph"]["edges"])
e["node_count"]=len(e["graph"]["nodes"]); e["edge_count"]=len(e["graph"]["edges"])
json.dump(e,open(p,"w"))
PY
  # 2. map (spoke-fabric link, ungated so a bad KB still produces output)
  if ! "$FABRIC" link "$exp" --out "$ann" --report "$rep" >>"$errlog" 2>&1; then
    echo "  FABRIC LINK FAILED (see $errlog)"; continue
  fi
  echo "  ok"
done
echo "done $(date +%H:%M:%S)"
