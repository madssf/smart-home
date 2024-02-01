import type {PriceInfo} from "~/routes/types";
import {displayPriceLevel, PriceLevel} from "~/routes/types";

export const capitalizeAndRemoveUnderscore = (str: string) => {
    if (str.length === 0) {
        return str.toUpperCase()
    }
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
    return `${priceInfo.amount.toFixed(2)} ${priceInfo.currency} - ${priceInfo.price_level ? displayPriceLevel(priceInfo.price_level) : `${displayPriceLevel(priceInfo.ext_price_level)} - ext.`}`;
};

export const formatNumber = (num: number, maximumFractionDigits?: number, minimumFractionDigits?: number) => {
    return Intl.NumberFormat('en', {maximumFractionDigits, minimumFractionDigits}).format(num);
};

const userLocale =
    typeof navigator === 'undefined' ? 'en' :
        navigator.languages && navigator.languages.length
            ? navigator.languages[0]
            : navigator.language;
