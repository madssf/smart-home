import {useNavigation} from "react-router";

export const useSubmissionStatus = (document: {id: string} | undefined) => {
    const navigation = useNavigation();
    const isCreating = navigation.formData?.get("intent") === "create" &&
        (navigation.formData?.get('id') ?? undefined) === document?.id;
    const isUpdating = navigation.formData?.get("intent") === "update" &&
        (navigation.formData?.get('id') ?? undefined) === document?.id;
    const isDeleting = navigation.formData?.get("intent") === "delete" &&
        (navigation.formData?.get('id') ?? undefined) === document?.id;
    const isNew = !document;
    return {isCreating, isDeleting, isUpdating, isNew};
};
