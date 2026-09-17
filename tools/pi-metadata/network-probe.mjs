const signal = AbortSignal.timeout(2000);
try {
  await fetch("http://1.1.1.1/", { signal });
  console.error("HEE3_NETWORK_UNEXPECTED_SUCCESS");
  process.exit(1);
} catch {
  console.log("HEE3_NETWORK_DENIED_CONTROL");
}
