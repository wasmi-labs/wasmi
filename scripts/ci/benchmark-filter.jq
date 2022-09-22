def round(p):
  if
    . != null
  then
    .*(100/p) | round | ./(1/p) | if . > 0 then "+\(.)" else . end
  else
    empty
  end;

def perf_to_emoji(p):
  if
    p == "Improved"
  then
    ":green_circle: "
  elif
    p == "Regressed"
  then
    ":red_circle: "
  elif
    p == "NoChange"
  then
    ":white_circle: "
  else
    empty
  end;

group_by(.id)
  | .[]
  | select(.[0].id != null)
  | "| \(.[0].id) " +
  "| \(.[0].slope.estimate) \(.[0].slope.unit) " +
  "| \(.[1].slope.estimate) \(.[1].slope.unit) " +
  "| \(perf_to_emoji(.[1].change.change)) \(.[1].change.mean.estimate | round(0.001)) % |\n"
