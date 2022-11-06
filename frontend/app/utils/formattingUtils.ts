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

export const formatNumber = (num: number, maximumFractionDigits?: number, minimumFractionDigits?: number) => {
    return Intl.NumberFormat('en', {maximumFractionDigits, minimumFractionDigits }).format(num);
};
