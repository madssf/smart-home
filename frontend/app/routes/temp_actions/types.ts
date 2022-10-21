
export interface TempAction {
    id: string,
    room_ids: string[],
    action_type: ActionType,
    expires_at: string,
}

export enum ActionType {
    ON = 'ON',
    OFF = 'OFF',
}
