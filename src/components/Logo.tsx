// A marca: um caminho que se ramifica, com a ponta escolhida preenchida e
// acima das alternativas. É o minimax do draft desenhado — prever qual galho
// sobrevive à resposta do adversário.
//
// Lê como sigilo a 16px e como árvore de decisão a 128px, que é o que um ícone
// precisa fazer.

export default function Logo({ size = 28 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="Presage"
      role="img"
    >
      <g
        stroke="currentColor"
        strokeWidth="5"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M32 57V38" />
        <path d="M32 38V17" />
        <path d="M32 38 16.5 25" opacity="0.5" />
        <path d="M32 38 47.5 25" opacity="0.5" />
      </g>
      <circle cx="32" cy="11" r="6" fill="currentColor" />
      <circle cx="14" cy="22" r="4.5" fill="currentColor" opacity="0.5" />
      <circle cx="50" cy="22" r="4.5" fill="currentColor" opacity="0.5" />
    </svg>
  );
}
