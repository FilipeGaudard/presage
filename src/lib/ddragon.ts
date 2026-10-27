const DDRAGON = "https://ddragon.leagueoflegends.com";

export interface Champion {
  id: number;
  key: string;
  name: string;
  iconUrl: string;
}

export interface StaticData {
  version: string;
  champions: Map<number, Champion>;
  allIds: number[];
}

interface DdragonChampionEntry {
  key: string;
  id: string;
  name: string;
}

export async function loadStaticData(locale = "pt_BR"): Promise<StaticData> {
  const versions: string[] = await fetchJson(`${DDRAGON}/api/versions.json`);
  const version = versions[0];

  let payload: { data: Record<string, DdragonChampionEntry> };
  try {
    payload = await fetchJson(`${DDRAGON}/cdn/${version}/data/${locale}/champion.json`);
  } catch {
    payload = await fetchJson(`${DDRAGON}/cdn/${version}/data/en_US/champion.json`);
  }

  const champions = new Map<number, Champion>();
  for (const entry of Object.values(payload.data)) {
    const id = Number(entry.key);
    champions.set(id, {
      id,
      key: entry.id,
      name: entry.name,
      iconUrl: `${DDRAGON}/cdn/${version}/img/champion/${entry.id}.png`,
    });
  }

  return {
    version,
    champions,
    allIds: [...champions.keys()].sort((a, b) => a - b),
  };
}

export function itemIcon(version: string, itemId: number): string {
  return `${DDRAGON}/cdn/${version}/img/item/${itemId}.png`;
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${url} respondeu ${response.status}`);
  }
  return response.json() as Promise<T>;
}
