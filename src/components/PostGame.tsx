import GraficoPartida from "./GraficoPartida";
import type { CoachNote, CoachReport } from "../lib/types";

interface Props {
  report: CoachReport | null;
}

function NoteCard({ note }: { note: CoachNote }) {
  return (
    <div className={`note-card ${note.verdict}`}>
      <b>{note.title}</b>
      <p>{note.detail}</p>
    </div>
  );
}

export default function PostGame({ report }: Props) {
  if (!report) {
    return (
      <div className="empty">
        <h2>Nenhuma partida lida ainda</h2>
        <p>
          O Presage grava a partida enquanto ela acontece e faz a leitura quando
          ela acaba.
        </p>
      </div>
    );
  }

  const vazio = report.good.length === 0 && report.bad.length === 0;

  return (
    <div className="postgame">
      <div className="panel-head">
        <div>
          <h2>
            {report.champion || "Última partida"}
            {report.won !== null && (
              <span className={`outcome ${report.won ? "good" : "bad"}`}>
                {report.won ? "vitória" : "derrota"}
              </span>
            )}
          </h2>
          {/* O resumo já abre com a duração; repetir aqui seria dizer duas vezes. */}
          <span className="muted">{report.summary}</span>
        </div>
      </div>

      {vazio ? (
        <p className="muted">Nada a apontar nesta partida.</p>
      ) : (
        <div className="notes">
          {report.bad.length > 0 && (
            <div className="notes-col">
              <h3 className="section-h">A corrigir</h3>
              {report.bad.map((note) => (
                <NoteCard key={note.title} note={note} />
              ))}
            </div>
          )}
          {report.good.length > 0 && (
            <div className="notes-col">
              <h3 className="section-h">A manter</h3>
              {report.good.map((note) => (
                <NoteCard key={note.title} note={note} />
              ))}
            </div>
          )}
        </div>
      )}

      <GraficoPartida series={report.series} duration={report.duration} />

      <p className="faint">
        Tudo aqui sai da gravação da própria partida, feita na sua máquina. Nada
        é enviado para lugar nenhum.
      </p>
    </div>
  );
}
