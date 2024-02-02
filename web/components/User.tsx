import { faTrash, faUser } from "@fortawesome/free-solid-svg-icons";
import { useDeleteUser, useUpdateUser, useUser } from "../api";
import { Card } from "./Card";
import { Error, Loading } from "./Placeholder";
import { Scrollable } from "./Scrollable";
import { useRef } from "react";
import { useNavigate } from "react-router-dom";

export function User() {
    const navigate = useNavigate();
    const { data, error } = useUser();
    const { mutate: update, isLoading: updateIsLoading } = useUpdateUser();
    const { mutate: delUser, isLoading: deleteIsLoading } = useDeleteUser();
    const usernameInput = useRef<HTMLInputElement>(null);
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    return (
        <Scrollable>
            <div className="card_stack card_stack--centred">
                <Card title={`Hi, ${data.displayName || "Anonymous"}!`} icon={faUser}>
                    Your account (ID {data.id}) was created on{" "}
                    {new Date(data.createdAt).toLocaleDateString()}.
                </Card>
                <Card title="Change display name">
                    <form className="form_row">
                        <input
                            type="text"
                            className="input"
                            defaultValue={data.displayName}
                            ref={usernameInput}
                        />
                        <button
                            type="submit"
                            className="submit"
                            onClick={() =>
                                update({ displayName: usernameInput.current?.value })
                            }
                            disabled={updateIsLoading}
                        >
                            { updateIsLoading ? "..." : "Update" }
                        </button>
                    </form>
                </Card>
                <Card
                    title={deleteIsLoading ? "Deleting..." : "Delete account"}
                    icon={faTrash}
                    onClick={async () => {
                        if (
                            !deleteIsLoading &&
                            confirm("Are you sure you want to delete your account?")
                        ) {
                            await delUser(null);
                            navigate("/");
                        }
                    }}
                    bad
                >
                    This is irreversible! You will lose all your games and data.
                </Card>
            </div>
        </Scrollable>
    );
}
