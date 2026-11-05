// Scenarios for the Início screen, on top of the fixtures from the real backend. Each one is an
// init script: it runs in the page, so `build` must not use anything from this module's scope.
function build(name) {
  const date = "2026-11-03"; // a Tuesday
  const when = (d, t) => ({ date: d, time: t ? `${t}:00` : null, at: t ? `${d}T${t}:00Z` : null });
  const stamp = (d, t = "11:00") => `${d}T${t}:00Z`;
  const item = (kind, id, title, w, o = {}) => ({ kind, id, board: "conteudo", title, when: w, end: null, network: null, status: null, column: null, done: false, ...o });
  const post = (id, title, w, network, o = {}) => item("publication", id, title, w, { network, status: "scheduled", column: "scheduled", ...o });
  const due = (id, title, w, o = {}) => item("due", id, title, w, { board: "tcc2", ...o });
  const event = (id, title, w, o = {}) => item("event", id, title, w, { board: null, ...o });
  const long = "Um título realmente muito longo de cartão que precisa caber em uma só linha com reticências sem empurrar nada para fora do widget";
  const solid = "TituloSemEspacosQueNaoPodeAlargarAColuna".repeat(4);
  const LEAD = "01JBX3K9Q2AB"; // real cards of the fixture, so the editor can open them
  const SECOND = "01JBX3K9Q3CD";
  const EVENT = "01JBY7P0D5EF";

  const cadence = (network, label, done, scheduled) => ({ network, label, goal: 2, per: "week", done, scheduled, period_start: "2026-11-02", period_end: "2026-11-08" });

  function typical(fx) {
    const base = fx.index.cards[0];
    const card = (o) => ({
      ...base, kind: "task", tags: [], network: null, publication: null, due: null, links: { board: null, siblings: [], article: null },
      created: null, updated: null, column_since: null, metrics: null, art: null, assets: [], body: "", checklist: { done: 0, total: 0 },
      hash: "h", rank: "m", path: `/x/${o.id}.md`, ...o,
    });
    const project = (id, name) => ({ ...fx.index.boards.find((b) => b.kind === "project"), id, name });
    const boards = [fx.index.boards.find((b) => b.kind === "content"), project("tcc2", "TCC2"), project("dungeon-meister", "Dungeon Meister"), project("pauta", "Pauta"), project("abstract-tex", "Abstract-Tex")];
    const ideas = [
      card({ id: "idea-1", board: "conteudo", column: "idea", title: "Thread sobre ownership para iniciantes", created: stamp("2026-11-02") }),
      card({ id: "idea-2", board: "conteudo", column: "idea", title: "Bastidores do TCC em três posts", created: stamp("2026-11-01") }),
      card({ id: "idea-3", board: "conteudo", column: "idea", title: "O que aprendi com o Pauta", created: stamp("2026-10-30") }),
      card({ id: "idea-4", board: "conteudo", column: "idea", title: "Ideia antiga", created: stamp("2026-10-01") }),
    ];
    const work = [
      card({ id: "t1", board: "tcc2", column: "review", title: "Revisar capítulo 3", created: stamp("2026-10-01"), updated: stamp(date), column_since: stamp(date) }),
      card({ id: "t2", board: "tcc2", column: "todo", title: "Entrega parcial do TCC2", due: when("2026-11-04"), created: stamp("2026-10-01"), updated: stamp("2026-10-20"), column_since: stamp("2026-10-20") }),
      card({ id: "d1", board: "dungeon-meister", column: "doing", title: "Mapa da masmorra", created: stamp("2026-10-01"), updated: stamp("2026-11-02"), column_since: stamp("2026-10-20") }),
      card({ id: "p1", board: "pauta", column: "done", title: "Redesenho do caderno", created: stamp("2026-10-01"), updated: stamp("2026-11-01"), column_since: stamp("2026-11-01") }),
      card({ id: "p2", board: "pauta", column: "doing", title: "Tela Início", created: stamp("2026-10-01"), updated: stamp("2026-10-30"), column_since: stamp("2026-10-30") }),
      card({ id: "a1", board: "abstract-tex", column: "backlog", title: "Ajustar o modelo do resumo", created: stamp("2026-10-28"), updated: stamp("2026-10-28"), column_since: stamp("2026-10-28") }),
    ];
    const index = {
      ...fx.index,
      boards,
      cards: [...fx.index.cards.filter((c) => c.board === "conteudo" && c.column !== "idea"), ...ideas, ...work],
      invalid: [],
      readiness: { "post-3": [{ code: "media_missing", detail: null }] },
    };
    const posts = [
      post(LEAD, "Por que Rust não tem coletor de lixo", when(date, "09:00"), "linkedin"),
      post(SECOND, "Carrossel: 5 mitos sobre IA generativa", when(date, "18:30"), "instagram"),
    ];
    const today = {
      date, time: "08:30:00", posts,
      overdue: [due(LEAD, "Revisar capítulo 3 do TCC2", when("2026-11-02"))],
      upcoming: [due("t2", "Entrega parcial do TCC2", when("2026-11-04"))],
      empty_queues: [{ network: "instagram", label: "Instagram", days: 7, candidates: [] }],
      stale: [
        { id: "s1", board: "conteudo", title: "Parado um", column: "draft", network: null },
        { id: "s2", board: "conteudo", title: "Parado dois", column: "draft", network: null },
      ],
      cadence: [cadence("linkedin", "LinkedIn", 1, 1), cadence("instagram", "Instagram", 0, 0)],
      invalid_files: 0,
    };
    const agenda = [
      event("week", "Semana de provas", when("2026-11-01"), { end: when("2026-11-05") }),
      ...posts,
      due("t0", "Enviar o rascunho ao orientador", when(date)),
      due("gone", "Prazo já cumprido", when(date), { done: true }),
      event(EVENT, "Reunião com o orientador", when(date, "14:00")),
      due("t2", "Entrega parcial do TCC2", when("2026-11-04")),
      post("post-3", "Bastidores do TCC em três posts", when("2026-11-04", "09:00"), "linkedin"),
      post("post-4", "Post sem problema", when("2026-11-05", "18:30"), "instagram"),
      event("banca", "Banca de qualificação", when("2026-11-09", "14:00")),
    ];
    return { index, today, agenda };
  }

  function full(fx) {
    const t = typical(fx);
    const titles = [long, solid];
    const pick = (i) => titles[i % 2];
    t.today.time = "08:30:00";
    t.today.posts = [
      post(LEAD, long, when(date, "09:00"), "linkedin"),
      post(SECOND, solid, when(date, "12:00"), "instagram"),
      post("post-a", long, when(date, "15:00"), "linkedin"),
      post("post-b", long, when(date, "18:00"), "instagram"),
      post("post-c", "Já publicado", when(date, "07:00"), "linkedin", { status: "published", done: true }),
    ];
    t.index.readiness = {
      [LEAD]: [{ code: "media_missing", detail: null }, { code: "no_text", detail: null }],
      "soon-1": [{ code: "over_limit", detail: "3000" }],
      "soon-2": [{ code: "art_pending", detail: "waiting" }],
      "soon-3": [{ code: "no_text", detail: null }],
    };
    t.today.overdue = Array.from({ length: 8 }, (_, i) => due(`late-${i}`, pick(i), when("2026-10-30")));
    t.today.empty_queues = [
      { network: "linkedin", label: "LinkedIn", days: 7, candidates: [] },
      { network: "instagram", label: "Instagram", days: 7, candidates: [] },
    ];
    t.today.stale = Array.from({ length: 12 }, (_, i) => ({ id: `stale-${i}`, board: "conteudo", title: "Parado", column: "draft", network: null }));
    t.today.invalid_files = 2;
    t.index.invalid = [
      { path: "/x/um.md", error: "cabeçalho quebrado" },
      { path: "/x/dois.md", error: "cabeçalho quebrado" },
    ];
    t.today.cadence = [cadence("linkedin", "LinkedIn", 1, 1), cadence("instagram", "Instagram", 0, 0), { ...cadence("mastodon", "Mastodon", 0, 1), goal: 12 }];
    t.agenda = [
      ...t.today.posts,
      post("soon-1", long, when("2026-11-04", "09:00"), "linkedin"),
      post("soon-2", long, when("2026-11-05", "09:00"), "instagram"),
      post("soon-3", long, when("2026-11-06", "09:00"), "linkedin"),
      ...Array.from({ length: 27 }, (_, i) => due(`a-${i}`, pick(i), when(`2026-11-${String(4 + (i % 7)).padStart(2, "0")}`))),
    ];
    const proto = t.index.boards[1];
    t.index.boards = [...t.index.boards, { ...proto, id: "extra-1", name: "Projeto com um nome realmente muito longo que não cabe" }, { ...proto, id: "extra-2", name: solid }];
    const card = t.index.cards.find((c) => c.id === "t1");
    t.index.cards = [
      ...t.index.cards.map((c) => (c.board === "tcc2" ? { ...c, title: long } : c)),
      { ...card, id: "x1", board: "extra-1", title: solid, column: "doing" },
      { ...card, id: "x2", board: "extra-2", title: long, column: "doing" },
      ...Array.from({ length: 9 }, (_, i) => ({ ...card, id: `i-${i}`, board: "conteudo", column: "idea", title: pick(i), created: stamp("2026-10-20") })),
    ];
    return t;
  }

  function empty(fx) {
    const t = typical(fx);
    t.index = { ...t.index, cards: [], events: [], readiness: {}, invalid: [] };
    t.today = { date, time: "20:00:00", posts: [], overdue: [], upcoming: [], empty_queues: [], stale: [], cadence: [], invalid_files: 0 };
    t.agenda = [];
    return t;
  }

  const scenarios = {
    typical,
    full,
    empty,
    // The next post's time has passed, so it is also in `overdue`.
    late(fx) {
      const t = typical(fx);
      t.today.time = "10:00:00";
      t.today.overdue = [post(LEAD, "Por que Rust não tem coletor de lixo", when(date, "09:00"), "linkedin")];
      return t;
    },
    noPosts(fx) {
      const t = typical(fx);
      t.today.posts = [];
      t.agenda = t.agenda.filter((i) => !(i.kind === "publication" && i.when.date === date));
      return t;
    },
    allPosted(fx) {
      const t = typical(fx);
      t.today.posts = t.today.posts.map((p) => ({ ...p, done: true, status: "published" }));
      return t;
    },
    noContentBoard(fx) {
      const t = typical(fx);
      t.index = { ...t.index, boards: t.index.boards.filter((b) => b.kind !== "content"), cards: t.index.cards.filter((c) => c.board !== "conteudo") };
      return t;
    },
  };

  let data;
  window.__calls = [];
  window.__override = (cmd, args) => {
    window.__calls.push({ cmd, args });
    data ??= scenarios[name](window.__fixtures);
    switch (cmd) {
      case "get_today": return window.__today ?? data.today;
      case "get_index": return data.index;
      case "get_agenda": return data.agenda;
      case "mark_published": return { id: args.id, kind: "post" };
      case "create_card": return window.__createCard ? window.__createCard(args) : { id: "new", title: args.card.title };
      default: return undefined;
    }
  };
}

const scenario = (name) => ({ content: `(${build})(${JSON.stringify(name)})` });

export const typicalHome = scenario("typical");
export const fullHome = scenario("full");
export const emptyHome = scenario("empty");
export const lateHome = scenario("late");
export const noPostsHome = scenario("noPosts");
export const allPostedHome = scenario("allPosted");
export const noContentBoardHome = scenario("noContentBoard");
