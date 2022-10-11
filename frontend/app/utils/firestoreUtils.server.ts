export const collections = {
    schedules: (userId: string) => generateCollectionPath(userId, 'schedules'),
    plugs: (userId: string) => generateCollectionPath(userId, 'plugs'),
    temp_actions: (userId: string) => generateCollectionPath(userId, 'temp_actions')
}

const generateCollectionPath = (userId: string, collection: string) => {
    return `users/${userId}/${collection}`
}
