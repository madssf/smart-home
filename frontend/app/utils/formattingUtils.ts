export const capitalizeAndRemoveUnderscore = (str: string) => {
    if (str.length === 0) {return str.toUpperCase()}
    return (str[0].toUpperCase() + str.slice(1).toLowerCase()).replace('_', ' ');
};
