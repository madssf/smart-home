
export interface TempAction {
    id: string,
    room_ids: string[],
    action: ActionType,
    temp: number | null,
    expires_at: string,
}

export enum ActionType {
    ON = 'ON',
    OFF = 'OFF',
}
