# Function to format percentage representation
# 0.1005511625312756 -> +10.0551
# -0.016916528214388715 -> -1.6917
# d - benchmark data, p - rounding precision
def format_percentage(d; p):
  if
    d != null
  then
    d*(100/p) | round | ./(1/p) | if . > 0 then "+\(.)" else . end
  else
    "no data"
  end;

# Function to format benchmark execution time
# 754.6489236864774 -> 754.6489 ns
# 51944.28340824702 -> 5194.428 µs
# 4033310.2747474746 -> 4.0333 ms
# d - benchmark data, p - rounding precision
def format_time(d; p):
  if
    d == null
  then
    "no data"
  elif
    d < 1000
  then
    d*(1/p) | round | ./(1/p) | "\(.) ns"
  elif
    d < 1000000
  then
    d/1000*(1/p) | round | ./(1/p) | "\(.) µs"
  else
    d/1000000*(1/p) | round | ./(1/p) | "\(.) ms"
  end;

# Map performance change to markdown emoji
def perf_to_emoji(d):
  if
    d == "Improved"
  then
    ":green_circle: "
  elif
    d == "Regressed"
  then
    ":red_circle: "
  elif
    d == "NoChange"
  then
    ":white_circle: "
  else
    empty
  end;

# main filter
group_by(.id)
  | .[]
  | select(.[0].id != null)
  | "| `\(.[0].id)` " +
  "| \(format_time(.[0].slope.estimate; 0.0001)) " +
  "| \(format_time(.[1].slope.estimate; 0.0001)) " +
  "| \(perf_to_emoji(.[1].change.change)) \(format_percentage(.[1].change.mean.estimate; 0.0001)) % |\n"
