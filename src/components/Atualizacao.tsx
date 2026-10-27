import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

/**
 * Aviso de versão nova.
 *
 * Duas regras, as duas pensando em quem está jogando:
 *
 * 1. **Nunca instala sozinho.** Trocar o executável e reiniciar no meio de um
 *    draft de 30 segundos seria o pior momento possível. O app avisa; quem
 *    decide a hora é a pessoa.
 * 2. **Falha calada.** Sem internet, sem servidor no ar, endpoint ainda não
 *    configurado — nada disso é problema de quem só quer jogar. O aviso
 *    simplesmente não aparece.
 */
export default function Atualizacao() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [estado, setEstado] = useState<"ocioso" | "baixando" | "pronto">("ocioso");
  const [pct, setPct] = useState(0);
  const [dispensado, setDispensado] = useState(false);

  useEffect(() => {
    let vivo = true;
    // Uma checagem na abertura. Ficar perguntando de minuto em minuto gasta
    // rede de quem está com o jogo aberto e não adianta nada.
    check()
      .then((achado) => {
        if (vivo && achado) setUpdate(achado);
      })
      .catch(() => void 0);
    return () => {
      vivo = false;
    };
  }, []);

  if (!update || dispensado) return null;

  async function instalar() {
    if (!update) return;
    setEstado("baixando");
    let baixado = 0;
    let total = 0;

    try {
      await update.downloadAndInstall((evento) => {
        if (evento.event === "Started") {
          total = evento.data.contentLength ?? 0;
        } else if (evento.event === "Progress") {
          baixado += evento.data.chunkLength;
          if (total > 0) setPct(Math.round((baixado / total) * 100));
        }
      });
      setEstado("pronto");
      await relaunch();
    } catch {
      // Não adianta explicar erro de rede para quem quer jogar. O aviso some
      // e volta na próxima abertura.
      setDispensado(true);
    }
  }

  return (
    <div className="update">
      <div className="update-texto">
        <b>Versão {update.version} disponível</b>
        {update.body && <span>{update.body}</span>}
      </div>

      {estado === "ocioso" ? (
        <>
          <button type="button" className="update-ok" onClick={instalar}>
            Atualizar e reiniciar
          </button>
          <button
            type="button"
            className="update-depois"
            onClick={() => setDispensado(true)}
            title="Volta a avisar na próxima vez que abrir"
          >
            Depois
          </button>
        </>
      ) : (
        <span className="update-progresso">
          {estado === "pronto" ? "Reiniciando…" : `Baixando ${pct}%`}
        </span>
      )}
    </div>
  );
}
