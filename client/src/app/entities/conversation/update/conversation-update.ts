import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { TypeChat } from 'app/entities/enumerations/type-chat.model';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { IConversation } from '../conversation.model';
import { ConversationService } from '../service/conversation.service';

import { ConversationFormGroup, ConversationFormService } from './conversation-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-conversation-update',
  templateUrl: './conversation-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class ConversationUpdate implements OnInit {
  readonly isSaving = signal(false);
  conversation: IConversation | null = null;
  typeChatValues = Object.keys(TypeChat);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected conversationService = inject(ConversationService);
  protected conversationFormService = inject(ConversationFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: ConversationFormGroup = this.conversationFormService.createConversationFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ conversation }) => {
      this.conversation = conversation;
      if (conversation) {
        this.updateForm(conversation);
      }
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const conversation = this.conversationFormService.getConversation(this.editForm);
    if (conversation.id === null) {
      this.subscribeToSaveResponse(this.conversationService.create(conversation));
    } else {
      this.subscribeToSaveResponse(this.conversationService.update(conversation));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IConversation | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(conversation: IConversation): void {
    this.conversation = conversation;
    this.conversationFormService.resetForm(this.editForm, conversation);
  }
}
