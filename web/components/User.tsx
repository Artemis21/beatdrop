import { faTrash, faUser } from "@fortawesome/free-solid-svg-icons";
import { User, useDeleteUser, useUpdateUser, useUser } from "../api";
import { Card } from "./Card";
import { Error, Loading } from "./Placeholder";
import { Scrollable } from "./Scrollable";
import { useRef } from "react";
import { useNavigate } from "react-router-dom";

export function User() {
    const { data, error } = useUser();
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    return (
        <Scrollable>
            <div className="card_stack card_stack--centred">
                <Card title={`Hi, ${data.displayName || "Anonymous"}!`} icon={faUser}>
                    Your account (ID {data.id}) was created on{" "}
                    {new Date(data.createdAt).toLocaleDateString()}.
                </Card>
                <UpdateUser current={data} />
                <DeleteUser />
            </div>
        </Scrollable>
    );
}

function UpdateUser({ current }: { current: User }) {
    const { mutate, isLoading } = useUpdateUser();
    const usernameInput = useRef<HTMLInputElement>(null);
    return (
        <Card title="Change display name">
            <form className="form_row">
                <input
                    type="text"
                    className="input"
                    defaultValue={current.displayName}
                    ref={usernameInput}
                />
                <button
                    type="submit"
                    className="submit"
                    onClick={() => mutate({ displayName: usernameInput.current?.value })}
                    disabled={isLoading}
                >
                    {isLoading ? "..." : "Update"}
                </button>
            </form>
        </Card>
    );
}

function DeleteUser() {
    const { mutate, isLoading } = useDeleteUser();
    const navigate = useNavigate();
    return (
        <Card
            title={isLoading ? "Deleting..." : "Delete account"}
            icon={faTrash}
            onClick={async () => {
                if (
                    !isLoading &&
                    confirm("Are you sure you want to delete your account?")
                ) {
                    await mutate(null);
                    navigate("/");
                }
            }}
            bad
        >
            This is irreversible! You will lose all your games and data.
        </Card>
    );
}
