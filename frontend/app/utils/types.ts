export type Validate<T> = {
    valid: true
    data: T
} | {
    valid: false
    error: string
}

export type FormErrors<T> = {
    [k in keyof T]?: string;
} & {
    other?: string
};
