import type {PriceInfo} from "~/routes/types";
import {PriceLevel} from "~/routes/types";

export const capitalizeAndRemoveUnderscore = (str: string) => {
    if (str.length === 0) {return str.toUpperCase()}
    return (str[0].toUpperCase() + str.slice(1).toLowerCase()).replace('_', ' ');
};

export const formatPriceLevel = (priceLevel: PriceLevel) => {
    switch (priceLevel) {
        case PriceLevel.VeryCheap:
            return 'Very cheap';
        case PriceLevel.Cheap:
            return 'Cheap';
        case PriceLevel.Normal:
            return 'Normal';
        case PriceLevel.Expensive:
            return 'Expensive';
        case PriceLevel.VeryExpensive:
            return 'Very expensive';
    }
};

export const formatCurrency = (num: number, currency: string) => {
    return Intl.NumberFormat(userLocale, {currency: currency, style: 'currency'}).format(num);
};



export const formatPriceInfo = (priceInfo: PriceInfo) => {
    // eslint-disable-next-line max-len
    return `${priceInfo.amount.toFixed(2)} ${priceInfo.currency} - ${formatPriceLevel(priceInfo.price_level ?? priceInfo.ext_price_level)}${priceInfo.price_level ? '': ' [EXT]'}`;
};

export const formatNumber = (num: number, maximumFractionDigits?: number, minimumFractionDigits?: number) => {
    return Intl.NumberFormat('en', {maximumFractionDigits, minimumFractionDigits }).format(num);
};

const userLocale =
    typeof navigator === 'undefined' ? 'en' :
    navigator.languages && navigator.languages.length
        ? navigator.languages[0]
        : navigator.language;
