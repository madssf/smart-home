export const capitalizeAndRemoveUnderscore = (str: string) => {
    if (str.length === 0) {return str.toUpperCase()}
    return (str[0].toUpperCase() + str.slice(1).toLowerCase()).replace('_', ' ');
};

export const formatNumber = (num: number, maximumFractionDigits?: number, minimumFractionDigits?: number) => {
    return Intl.NumberFormat('en', {maximumFractionDigits, minimumFractionDigits }).format(num);
};
