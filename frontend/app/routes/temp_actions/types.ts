
export interface TempAction {
    id: string,
    plug_ids: string[],
    action_type: ActionType,
    expires_at: string,
}

export enum ActionType {
    ON = 'ON',
    OFF = 'OFF',
}
