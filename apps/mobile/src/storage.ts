import * as SecureStore from "expo-secure-store";

const HOST_KEY = "friday_bridge_host";
const TOKEN_KEY = "friday_bridge_token";

export async function loadPairing(): Promise<{ host: string; token: string } | null> {
  const host = await SecureStore.getItemAsync(HOST_KEY);
  const token = await SecureStore.getItemAsync(TOKEN_KEY);
  if (!host || !token) return null;
  return { host, token };
}

export async function savePairing(host: string, token: string): Promise<void> {
  await SecureStore.setItemAsync(HOST_KEY, host.replace(/\/$/, ""));
  await SecureStore.setItemAsync(TOKEN_KEY, token);
}

export async function clearPairing(): Promise<void> {
  await SecureStore.deleteItemAsync(HOST_KEY);
  await SecureStore.deleteItemAsync(TOKEN_KEY);
}
