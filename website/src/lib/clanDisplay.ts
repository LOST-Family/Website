import type { GameType } from './auth';
import { badgeNameFromId } from './crUtils';

import banner3 from '../assets/Clans/Clash of Clans/Lost-X-3.png';
import banner4 from '../assets/Clans/Clash of Clans/Lost-X-4.png';
import banner5 from '../assets/Clans/Clash of Clans/Lost-X-5.png';
import banner6 from '../assets/Clans/Clash of Clans/Lost-X-6.png';
import banner7 from '../assets/Clans/Clash of Clans/Lost-X-7.png';
import banner8 from '../assets/Clans/Clash of Clans/Lost-X-8.png';
import bannerF2P from '../assets/Clans/Clash of Clans/Lost-X-f2p.png';
import bannerF2P2 from '../assets/Clans/Clash of Clans/Lost-X-f2p2.png';
import bannerGP from '../assets/Clans/Clash of Clans/Lost-X-gp.png';
import bannerAnthrazit from '../assets/Clans/Clash of Clans/Lost-X-anthrazit.png';
import bannerDefault from '../assets/Assets/banner-lost.png';

import bannerCR1 from '../assets/Clans/Clash Royale/Lost_1.png';
import bannerCR2 from '../assets/Clans/Clash Royale/Lost_2.png';
import bannerCR3 from '../assets/Clans/Clash Royale/Lost_3.png';
import bannerCR4 from '../assets/Clans/Clash Royale/Lost_4.png';
import bannerCR5 from '../assets/Clans/Clash Royale/Lost_5.png';

export function getClanBanner(clanName: string, gameType?: GameType): string {
    const name = (clanName || '').toUpperCase();

    if (gameType === 'cr') {
        if (name === 'LOST') return bannerCR1;
        if (name.includes('4') || name.includes('IV')) return bannerCR4;
        if (name.includes('5') || name.includes('V')) return bannerCR5;
        if (name.includes('3') || name.includes('III')) return bannerCR3;
        if (name.includes('2') || name.includes('II')) return bannerCR2;
        return bannerDefault;
    }

    if (name.includes('F2P 2') || name.includes('F2P2')) return bannerF2P2;
    if (name.includes('F2P')) return bannerF2P;
    if (name.includes('GP')) return bannerGP;
    if (name.includes('8') || name.includes('VIII')) return banner8;
    if (name.includes('7') || name.includes('VII')) return banner7;
    if (name.includes('6') || name.includes('VI')) return banner6;
    if (name.includes('4') || name.includes('IV')) return banner4;
    if (name.includes('5') || name.includes('V')) return banner5;
    if (name.includes('3') || name.includes('III')) return banner3;
    if (name.includes('ANTHRAZIT')) return bannerAnthrazit;
    return bannerDefault;
}

export function getClanColor(name: string, index: number): string {
    const n = (name || '').toUpperCase();
    if (n.includes('GP')) return '#a5025a';
    if (n.includes('ANTHRAZIT')) return '#3d3a3f';

    if (n.includes('F2P 2') || n.includes('F2P2')) return '#05762b';
    if (n.includes('F2P')) return '#c90000';
    if (n.includes('8') || n.includes('VIII')) return '#d100c7';
    if (n.includes('7') || n.includes('VII')) return '#007076';
    if (n.includes('6') || n.includes('VI')) return '#b54800';
    if (n.includes('4') || n.includes('IV')) return '#691a97';
    if (n.includes('5') || n.includes('V')) return '#024885';
    if (n.includes('3') || n.includes('III')) return '#c89e00';

    if (index === 1) return '#c90000';
    if (index === 2) return '#05762b';
    if (index === 3) return '#c89e00';
    if (index === 4) return '#691a97';
    if (index === 5) return '#024885';
    if (index === 6) return '#b54800';
    if (index === 7) return '#007076';
    if (index === 8) return '#d100c7';

    return '#c90000';
}

export function getClanBadgeUrl(clan: any): string | undefined {
    if (!clan) return undefined;
    if (clan.badgeId) {
        return `https://raw.githubusercontent.com/RoyaleAPI/cr-api-assets/master/badges/${badgeNameFromId(clan.badgeId)}.png`;
    }
    if (clan.badgeUrl) return clan.badgeUrl;
    if (clan.badgeUrls?.large) return clan.badgeUrls.large;
    if (clan.badgeUrls?.medium) return clan.badgeUrls.medium;
    if (clan.badgeUrls?.small) return clan.badgeUrls.small;
    return undefined;
}
