import fs from "node:fs";
const css = fs.readFileSync("src/styles.css", "utf8");
function variables(selector) {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const block = new RegExp(escaped + "\\s*\\{([^}]+)\\}").exec(css)?.[1];
  if (!block) throw new Error("Missing theme block: " + selector);
  return Object.fromEntries([...block.matchAll(/--([a-z-]+):\s*(#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{3}))\s*;?/g)].map((m) => {
    const color = m[2];
    const normalized = color.length === 4
      ? "#" + [...color.slice(1)].map((digit) => digit + digit).join("")
      : color;
    return [m[1], normalized];
  }));
}
function luminance(hex) {
  const channels = hex.slice(1).match(/../g).map((s) => parseInt(s, 16) / 255);
  const linear = channels.map((v) => v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
}
function contrast(a, b) {
  const x = luminance(a), y = luminance(b);
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}
const light = variables(".settings-page");
const dark = variables(".settings-page[data-theme=\"dark\"]");
const pairs = [
  ["settings-text", "settings-page-bg"], ["settings-text", "settings-card"],
  ["settings-label", "settings-card"], ["settings-muted", "settings-page-bg"],
  ["settings-small", "settings-card"], ["settings-accent", "settings-card"],
  ["settings-message-ok", "settings-card"], ["settings-message-warn", "settings-card"],
  ["settings-message-error", "settings-card"], ["settings-control-text", "settings-control-bg"],
  ["settings-label", "settings-input-bg"]
];
for (const [themeName, values] of [["light", light], ["dark", dark]]) {
  for (const [foreground, background] of pairs) {
    const fg = values[foreground], bg = values[background];
    if (!fg || !bg) throw new Error(`Missing ${foreground} or ${background} token in ${themeName} theme.`);
    const ratio = contrast(fg, bg);
    if (ratio < 4.5) throw new Error(`${themeName} theme contrast below WCAG AA: ${foreground} on ${background} = ${ratio.toFixed(2)}:1`);
  }
}
console.log("Theme contrast check passed: all tested text/token pairs meet 4.5:1 in light and dark modes.");
