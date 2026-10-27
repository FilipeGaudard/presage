import { OVERLAY_SECTIONS, type OverlayPrefs, type OverlaySection } from "../lib/types";

interface Props {
  prefs: OverlayPrefs;
  onChange: (prefs: OverlayPrefs) => void;
}

/**
 * O que aparece por cima do jogo.
 *
 * Não existe configuração certa aqui. Quem joga de suporte olha objetivo e
 * ignora build; quem joga de ADC quer o próximo item e acha o ouro do time
 * ruído. Por isso cada seção liga e desliga em vez de a gente escolher por
 * todo mundo.
 */
export default function OverlayPrefsPanel({ prefs, onChange }: Props) {
  const mostra = (id: OverlaySection) => !prefs.hidden.includes(id);
  const nenhuma = prefs.hidden.length >= OVERLAY_SECTIONS.length;

  function alternar(id: OverlaySection) {
    const hidden = mostra(id)
      ? [...prefs.hidden, id]
      : prefs.hidden.filter((s) => s !== id);
    onChange({ hidden });
  }

  return (
    <div className="prefs">
      <div className="panel-head">
        <div>
          <h2>O que aparece no overlay</h2>
          <span className="muted">
            Vale na hora, mesmo com a partida rodando. O painel continua
            mostrando tudo.
          </span>
        </div>
        {prefs.hidden.length > 0 && (
          <button type="button" className="ghost" onClick={() => onChange({ hidden: [] })}>
            Mostrar tudo
          </button>
        )}
      </div>

      <div className="prefs-lista">
        {OVERLAY_SECTIONS.map((secao) => (
          <label className={`pref ${mostra(secao.id) ? "on" : ""}`} key={secao.id}>
            <input
              type="checkbox"
              checked={mostra(secao.id)}
              onChange={() => alternar(secao.id)}
            />
            <span>
              <b>{secao.label}</b>
              <small>{secao.detail}</small>
            </span>
          </label>
        ))}
      </div>

      {nenhuma && (
        <p className="warn-line">
          Tudo desligado: o overlay não vai aparecer. O painel continua
          funcionando normalmente.
        </p>
      )}

      <p className="faint">
        O overlay abre e fecha com <b>Ctrl+Shift+P</b>. Ele nasce atravessável
        pelo mouse, então não come clique; <b>Ctrl+Shift+O</b> deixa a janela
        clicável quando você quiser arrastá-la.
      </p>
    </div>
  );
}
