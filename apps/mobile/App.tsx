import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ActivityIndicator,
  Button,
  FlatList,
  SafeAreaView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import * as Notifications from "expo-notifications";
import * as SecureStore from "expo-secure-store";
import { StatusBar } from "expo-status-bar";

import type { AgentEvent, FridaySession } from "@friday/agent-core";
import { FridayBridgeClient } from "@friday/bridge-client";

Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowAlert: true,
    shouldPlaySound: true,
    shouldSetBadge: false,
  }),
});

const BRIDGE_URL_KEY = "friday.bridge.url";
const BRIDGE_TOKEN_KEY = "friday.bridge.token";

export default function App() {
  const [baseUrl, setBaseUrl] = useState("");
  const [token, setToken] = useState("");
  const [paired, setPaired] = useState(false);
  const [loading, setLoading] = useState(true);
  const [sessions, setSessions] = useState<FridaySession[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [events, setEvents] = useState<AgentEvent[]>([]);
  const [error, setError] = useState<string | null>(null);

  const client = useMemo(() => {
    if (!paired || !baseUrl || !token) return null;
    return new FridayBridgeClient({ baseUrl, token });
  }, [paired, baseUrl, token]);

  useEffect(() => {
    void (async () => {
      const savedUrl = await SecureStore.getItemAsync(BRIDGE_URL_KEY);
      const savedToken = await SecureStore.getItemAsync(BRIDGE_TOKEN_KEY);
      if (savedUrl && savedToken) {
        setBaseUrl(savedUrl);
        setToken(savedToken);
        setPaired(true);
      }
      setLoading(false);
    })();
  }, []);

  const refreshSessions = useCallback(async () => {
    if (!client) return;
    try {
      setError(null);
      const list = await client.listSessions();
      setSessions(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load sessions");
    }
  }, [client]);

  const loadEvents = useCallback(
    async (sessionId: string) => {
      if (!client) return;
      setActiveId(sessionId);
      try {
        const list = await client.getEvents(sessionId);
        setEvents(list);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load events");
      }
    },
    [client],
  );

  useEffect(() => {
    if (!client) return;
    void refreshSessions();
    const ws = client.connectWebSocket((event) => {
      setEvents((prev) => [...prev.slice(-199), event]);
      if (event.type === "approval.required") {
        void Notifications.scheduleNotificationAsync({
          content: {
            title: "Friday — Approval needed",
            body: event.command ?? event.title,
          },
          trigger: null,
        });
      }
    });
    const interval = setInterval(() => void refreshSessions(), 8000);
    return () => {
      ws.close();
      clearInterval(interval);
    };
  }, [client, refreshSessions]);

  const pair = async () => {
    const probe = new FridayBridgeClient({ baseUrl: baseUrl.trim(), token: token.trim() });
    try {
      await probe.health();
      await probe.getInfo();
      await SecureStore.setItemAsync(BRIDGE_URL_KEY, baseUrl.trim());
      await SecureStore.setItemAsync(BRIDGE_TOKEN_KEY, token.trim());
      setPaired(true);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Pairing failed");
    }
  };

  if (loading) {
    return (
      <View style={styles.center}>
        <ActivityIndicator />
      </View>
    );
  }

  if (!paired) {
    return (
      <SafeAreaView style={styles.container}>
        <StatusBar style="light" />
        <Text style={styles.title}>Friday Remote</Text>
        <Text style={styles.hint}>Enter the bridge URL and token from desktop Settings → Mobile Remote.</Text>
        <TextInput
          style={styles.input}
          placeholder="http://192.168.1.10:8787"
          placeholderTextColor="#666"
          autoCapitalize="none"
          value={baseUrl}
          onChangeText={setBaseUrl}
        />
        <TextInput
          style={styles.input}
          placeholder="Auth token"
          placeholderTextColor="#666"
          autoCapitalize="none"
          secureTextEntry
          value={token}
          onChangeText={setToken}
        />
        {error && <Text style={styles.error}>{error}</Text>}
        <Button title="Pair" onPress={() => void pair()} />
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar style="light" />
      <View style={styles.row}>
        <Text style={styles.title}>Sessions</Text>
        <Button title="Refresh" onPress={() => void refreshSessions()} />
      </View>
      {error && <Text style={styles.error}>{error}</Text>}
      <FlatList
        data={sessions}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => (
          <View style={styles.card}>
            <Text style={styles.cardTitle}>{item.title}</Text>
            <Text style={styles.meta}>{item.status}</Text>
            <View style={styles.row}>
              <Button title="Watch" onPress={() => void loadEvents(item.id)} />
              <Button title="Stop" color="#c0392b" onPress={() => void client?.stopSession(item.id)} />
            </View>
          </View>
        )}
        ListEmptyComponent={<Text style={styles.hint}>No active sessions</Text>}
      />
      {activeId && (
        <View style={styles.timeline}>
          <Text style={styles.cardTitle}>Timeline</Text>
          {events.slice(-8).map((ev, i) => (
            <Text key={`${ev.timestamp}-${i}`} style={styles.meta}>
              {ev.type}
            </Text>
          ))}
        </View>
      )}
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#0a0a0a", padding: 16 },
  center: { flex: 1, alignItems: "center", justifyContent: "center", backgroundColor: "#0a0a0a" },
  title: { color: "#f4f4f5", fontSize: 20, fontWeight: "600", marginBottom: 8 },
  hint: { color: "#71717a", marginBottom: 12 },
  input: {
    backgroundColor: "#18181b",
    color: "#f4f4f5",
    borderRadius: 8,
    padding: 12,
    marginBottom: 10,
  },
  error: { color: "#f87171", marginBottom: 8 },
  row: { flexDirection: "row", alignItems: "center", justifyContent: "space-between", gap: 8 },
  card: { backgroundColor: "#18181b", borderRadius: 10, padding: 12, marginBottom: 10 },
  cardTitle: { color: "#e4e4e7", fontWeight: "600" },
  meta: { color: "#a1a1aa", fontSize: 12, marginTop: 4 },
  timeline: { borderTopColor: "#27272a", borderTopWidth: 1, paddingTop: 12, marginTop: 8 },
});
