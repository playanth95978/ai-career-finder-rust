import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { AutoApplyMode } from 'app/entities/enumerations/auto-apply-mode.model';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { IAutoApplyConfig } from '../auto-apply-config.model';
import { AutoApplyConfigService } from '../service/auto-apply-config.service';

import { AutoApplyConfigFormGroup, AutoApplyConfigFormService } from './auto-apply-config-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-auto-apply-config-update',
  templateUrl: './auto-apply-config-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class AutoApplyConfigUpdate implements OnInit {
  readonly isSaving = signal(false);
  autoApplyConfig: IAutoApplyConfig | null = null;
  autoApplyModeValues = Object.keys(AutoApplyMode);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected autoApplyConfigService = inject(AutoApplyConfigService);
  protected autoApplyConfigFormService = inject(AutoApplyConfigFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: AutoApplyConfigFormGroup = this.autoApplyConfigFormService.createAutoApplyConfigFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ autoApplyConfig }) => {
      this.autoApplyConfig = autoApplyConfig;
      if (autoApplyConfig) {
        this.updateForm(autoApplyConfig);
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
    const autoApplyConfig = this.autoApplyConfigFormService.getAutoApplyConfig(this.editForm);
    if (autoApplyConfig.id === null) {
      this.subscribeToSaveResponse(this.autoApplyConfigService.create(autoApplyConfig));
    } else {
      this.subscribeToSaveResponse(this.autoApplyConfigService.update(autoApplyConfig));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IAutoApplyConfig | null>): void {
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

  protected updateForm(autoApplyConfig: IAutoApplyConfig): void {
    this.autoApplyConfig = autoApplyConfig;
    this.autoApplyConfigFormService.resetForm(this.editForm, autoApplyConfig);
  }
}
