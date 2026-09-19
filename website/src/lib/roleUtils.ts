import type { GameType } from './auth';

export const ROLE_ORDER: Record<string, number> = {
    leader: 1,
    // Brawl Stars nennt dieselben Raenge anders, und zwar zweimal verschieden:
    // die Supercell-API sagt president / vicePresident / senior, der bsmanager
    // sagt PRESIDENT / COPRESIDENT / SENIOR. Beide kommen im Frontend an — die
    // Spielrolle aus der API, die Sollrolle aus dem Bot.
    president: 1,
    coLeader: 2,
    hiddencoleader: 2,
    vicePresident: 2,
    copresident: 2,
    admin: 3,
    elder: 3,
    senior: 3,
    member: 4,
};

export function getRoleDisplay(
    role: string | undefined,
    gameType: GameType = 'coc',
): string {
    switch (role?.toLowerCase()) {
        case 'leader':
        case 'president':
            return 'Anführer';
        case 'coleader':
        case 'vicepresident':
        case 'copresident':
            return 'Vize-Anführer';
        // Ein Vize ohne die Discord-Rolle: er hat die Rechte, taucht in der
        // Mitgliederliste aber nicht als Vize auf. Stand bisher roh als
        // „hiddencoleader" da, weil ihn niemand setzen konnte.
        case 'hiddencoleader':
            return 'Vize (versteckt)';
        case 'elder':
        case 'senior':
            return 'Ältester';
        case 'admin':
            // In Clash of Clans heisst der Aeltesten-Rang in der API "admin",
            // ein "elder" existiert dort nicht. In Clash Royale gibt es
            // beides: "elder" ist der echte Rang, "admin" dagegen die
            // Dummy-Marke aus dem Bot, damit ein Platzhalter nicht als
            // normales Mitglied zaehlt. Ihn als "Ältester" anzuzeigen war
            // falsch (Issue #23). In Brawl Stars ist es dasselbe: der
            // bsmanager setzt ADMIN, wenn der verknuepfte Nutzer Bot-Admin
            // ist, und ueberschreibt damit den echten Clubrang.
            return gameType === 'coc' ? 'Ältester' : 'Mitglied';
        case 'member':
            return 'Mitglied';
        default:
            return role ?? '';
    }
}

export function isRoleWrong(
    current: any,
    expected: any,
    cocMode: boolean = true,
): boolean {
    if (!expected) return false;

    const c = String(current || '')
        .toLowerCase()
        .trim();
    const e = String(expected || '')
        .toLowerCase()
        .trim();

    if (c === e) return false;

    const normalize = (r: string) => {
        if (cocMode && r === 'admin') return 'elder';
        if (r === 'elder' || r === 'ältester' || r === 'senior') return 'elder';
        if (
            r === 'coleader' ||
            r === 'co-leader' ||
            r === 'vicepresident' ||
            r === 'copresident' ||
            r === 'vize-anführer' ||
            r === 'vize'
        )
            return 'coleader';
        if (r === 'leader' || r === 'president' || r === 'anführer')
            return 'leader';
        if (r === 'member' || r === 'mitglied') return 'member';
        return r;
    };

    return normalize(c) !== normalize(e);
}

export function getPlayerName(p: {
    name?: string;
    nickname?: string;
    global_name?: string;
    username?: string;
    upstream_name?: string;
}): string {
    const nameCandidate = p.name || '';
    if (!nameCandidate || nameCandidate.startsWith('#')) {
        return (
            p.nickname ||
            p.global_name ||
            p.username ||
            p.upstream_name ||
            nameCandidate ||
            'Unbekannt'
        );
    }
    return nameCandidate;
}
