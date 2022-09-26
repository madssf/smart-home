export const collections = {
    schedules: (userId: string) => generateCollectionPath(userId, 'schedules'),
    plugs: (userId: string) => generateCollectionPath(userId, 'plugs')
}

const generateCollectionPath = (userId: string, collection: string) => {
    return `users/${userId}/${collection}`
}