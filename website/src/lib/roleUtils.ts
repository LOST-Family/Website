export const ROLE_ORDER: Record<string, number> = {
    leader: 1,
    coLeader: 2,
    admin: 3,
    elder: 3,
    member: 4,
};

export function getRoleDisplay(role: string): string {
    switch (role?.toLowerCase()) {
        case 'leader':
            return 'Anführer';
        case 'coleader':
            return 'Vize-Anführer';
        case 'admin':
        case 'elder':
            return 'Ältester';
        case 'member':
            return 'Mitglied';
        default:
            return role;
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
        if (r === 'elder' || r === 'ältester') return 'elder';
        if (
            r === 'coleader' ||
            r === 'co-leader' ||
            r === 'vize-anführer' ||
            r === 'vize'
        )
            return 'coleader';
        if (r === 'leader' || r === 'anführer') return 'leader';
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
