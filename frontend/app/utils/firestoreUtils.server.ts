export const collections = {
    schedules: (userId: string) => generateCollectionPath(userId, 'schedules'),
    plugs: (userId: string) => generateCollectionPath(userId, 'plugs'),
    tempActions: (userId: string) => generateCollectionPath(userId, 'temp_actions'),
    tempLog: (userId: string) => generateCollectionPath(userId, 'temperature_log'),
};

const generateCollectionPath = (userId: string, collection: string) => {
    return `users/${userId}/${collection}`;
};
